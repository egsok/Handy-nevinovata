use crate::actions::ACTION_MAP;
use crate::managers::audio::AudioRecordingManager;
use log::{debug, error, info, warn};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};

const DEBOUNCE: Duration = Duration::from_millis(30);

/// Commands processed sequentially by the coordinator thread.
enum Command {
    Input {
        binding_id: String,
        hotkey_string: String,
        is_pressed: bool,
        /// `None` = resolve from settings on the coordinator thread (keeps
        /// the settings store off the hotkey manager thread). `Some(false)`
        /// = signal/CLI toggles, which are always toggle-mode regardless of
        /// the user's push-to-talk setting.
        push_to_talk: Option<bool>,
    },
    Cancel {
        recording_was_active: bool,
    },
    ProcessingFinished,
    /// Bypasses the Processing-state guard that blocks `Cancel`. Used by the
    /// "Force Reset Pipeline" tray item to recover from a stuck Processing
    /// stage without restarting the app. See plan: hotkey debug session.
    ForceIdle,
}

/// Pipeline lifecycle, owned exclusively by the coordinator thread.
enum Stage {
    Idle,
    Recording(String), // binding_id
    Processing,
}

fn stage_label(stage: &Stage) -> &'static str {
    match stage {
        Stage::Idle => "Idle",
        Stage::Recording(_) => "Recording",
        Stage::Processing => "Processing",
    }
}

/// Mirror the stage into the watchdog so stall reports can say what the
/// pipeline was doing when a thread stopped responding.
fn publish_stage(stage: &Stage) {
    crate::watchdog::set_coordinator_stage(match stage {
        Stage::Idle => 0,
        Stage::Recording(_) => 1,
        Stage::Processing => 2,
    });
}

/// Serialises all transcription lifecycle events through a single thread
/// to eliminate race conditions between keyboard shortcuts, signals, and
/// the async transcribe-paste pipeline.
pub struct TranscriptionCoordinator {
    tx: Sender<Command>,
}

pub fn is_transcribe_binding(id: &str) -> bool {
    id == "transcribe" || id == "transcribe_with_post_process"
}

impl TranscriptionCoordinator {
    pub fn new(app: AppHandle) -> Self {
        let (tx, rx) = mpsc::channel();

        thread::spawn(move || {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let mut stage = Stage::Idle;
                let mut last_press: Option<Instant> = None;
                // When Some, we're currently in Stage::Processing — used to
                // measure how long Processing actually takes per cycle. This
                // is the key timing for diagnosing "hotkey ignored briefly".
                let mut processing_started: Option<Instant> = None;

                loop {
                    crate::watchdog::beat_coordinator();
                    let cmd = match rx.recv_timeout(Duration::from_secs(1)) {
                        Ok(cmd) => cmd,
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    };
                    match cmd {
                        Command::Input {
                            binding_id,
                            hotkey_string,
                            is_pressed,
                            push_to_talk,
                        } => {
                            // Resolve push-to-talk here, on the coordinator
                            // thread — reading the settings store from the
                            // hotkey manager thread risks stalling the hook.
                            let push_to_talk = push_to_talk.unwrap_or_else(|| {
                                crate::settings::get_settings(&app).push_to_talk
                            });

                            info!(
                                "coordinator: input(binding={}, pressed={}, ptt={}) | stage={}",
                                binding_id,
                                is_pressed,
                                push_to_talk,
                                stage_label(&stage)
                            );

                            // Debounce rapid-fire press events (key repeat / double-tap).
                            // Releases always pass through for push-to-talk.
                            if is_pressed {
                                let now = Instant::now();
                                if last_press.is_some_and(|t| now.duration_since(t) < DEBOUNCE) {
                                    debug!("Debounced press for '{binding_id}'");
                                    continue;
                                }
                                last_press = Some(now);
                            }

                            if push_to_talk {
                                if is_pressed && matches!(stage, Stage::Idle) {
                                    start(&app, &mut stage, &binding_id, &hotkey_string);
                                } else if !is_pressed
                                    && matches!(&stage, Stage::Recording(id) if id == &binding_id)
                                {
                                    stop(
                                        &app,
                                        &mut stage,
                                        &mut processing_started,
                                        &binding_id,
                                        &hotkey_string,
                                    );
                                }
                            } else if is_pressed {
                                match &stage {
                                    Stage::Idle => {
                                        start(&app, &mut stage, &binding_id, &hotkey_string);
                                    }
                                    Stage::Recording(id) if id == &binding_id => {
                                        stop(
                                            &app,
                                            &mut stage,
                                            &mut processing_started,
                                            &binding_id,
                                            &hotkey_string,
                                        );
                                    }
                                    _ => {
                                        info!(
                                            "coordinator: ignoring press for '{binding_id}' — \
                                             pipeline busy in stage={}",
                                            stage_label(&stage)
                                        );
                                    }
                                }
                            }
                        }
                        Command::Cancel {
                            recording_was_active,
                        } => {
                            // Don't reset during processing — wait for the pipeline to finish.
                            if !matches!(stage, Stage::Processing)
                                && (recording_was_active || matches!(stage, Stage::Recording(_)))
                            {
                                info!(
                                    "coordinator: stage Idle (was {}, via Cancel)",
                                    stage_label(&stage)
                                );
                                stage = Stage::Idle;
                                publish_stage(&stage);
                            } else {
                                debug!(
                                    "coordinator: Cancel ignored (stage={}, recording_was_active={})",
                                    stage_label(&stage),
                                    recording_was_active
                                );
                            }
                        }
                        Command::ProcessingFinished => {
                            let elapsed_ms = processing_started
                                .take()
                                .map(|t| t.elapsed().as_millis())
                                .unwrap_or(0);
                            info!("coordinator: stage Idle (Processing took {}ms)", elapsed_ms);
                            stage = Stage::Idle;
                            publish_stage(&stage);
                        }
                        Command::ForceIdle => {
                            let elapsed =
                                processing_started.take().map(|t| t.elapsed().as_millis());
                            info!(
                                "coordinator: ForceIdle (was stage={}, processing_elapsed={:?}ms)",
                                stage_label(&stage),
                                elapsed
                            );
                            stage = Stage::Idle;
                            publish_stage(&stage);
                        }
                    }
                }
                debug!("Transcription coordinator exited");
            }));
            if let Err(e) = result {
                error!("Transcription coordinator panicked: {e:?}");
            }
        });

        Self { tx }
    }

    /// Send a keyboard/signal input event for a transcribe binding.
    /// `push_to_talk`: `None` resolves the user's setting on the coordinator
    /// thread; signal/CLI toggles pass `Some(false)` (always toggle-mode).
    pub fn send_input(
        &self,
        binding_id: &str,
        hotkey_string: &str,
        is_pressed: bool,
        push_to_talk: Option<bool>,
    ) {
        if self
            .tx
            .send(Command::Input {
                binding_id: binding_id.to_string(),
                hotkey_string: hotkey_string.to_string(),
                is_pressed,
                push_to_talk,
            })
            .is_err()
        {
            warn!("Transcription coordinator channel closed");
        }
    }

    pub fn notify_cancel(&self, recording_was_active: bool) {
        if self
            .tx
            .send(Command::Cancel {
                recording_was_active,
            })
            .is_err()
        {
            warn!("Transcription coordinator channel closed");
        }
    }

    pub fn notify_processing_finished(&self) {
        if self.tx.send(Command::ProcessingFinished).is_err() {
            warn!("Transcription coordinator channel closed");
        }
    }

    /// Force the coordinator back to `Idle` state, bypassing the guard that
    /// normally blocks `Cancel` while in `Processing`. Used by the tray
    /// "Force Reset Pipeline" recovery action when the pipeline is stuck.
    pub fn force_idle(&self) {
        if self.tx.send(Command::ForceIdle).is_err() {
            warn!("Transcription coordinator channel closed");
        }
    }
}

/// Threshold above which an action's synchronous part counts as a stall.
/// `action.start` opens the mic stream and updates tray/overlay; `action.stop`
/// updates tray/overlay and spawns the async transcription. All of those are
/// normally well under 200ms.
const ACTION_STALL: Duration = Duration::from_millis(1000);

fn start(app: &AppHandle, stage: &mut Stage, binding_id: &str, hotkey_string: &str) {
    let Some(action) = ACTION_MAP.get(binding_id) else {
        warn!("No action in ACTION_MAP for '{binding_id}'");
        return;
    };
    let t = Instant::now();
    action.start(app, binding_id, hotkey_string);
    if t.elapsed() > ACTION_STALL {
        warn!(
            "coordinator: action.start blocked for {:?} — main-thread (tray/overlay) \
             or audio-device stall; hotkey presses queued during this time",
            t.elapsed()
        );
    }
    if app
        .try_state::<Arc<AudioRecordingManager>>()
        .is_some_and(|a| a.is_recording())
    {
        info!("coordinator: stage Recording (binding={binding_id})");
        *stage = Stage::Recording(binding_id.to_string());
    } else {
        debug!("Start for '{binding_id}' did not begin recording; staying idle");
    }
    publish_stage(stage);
}

fn stop(
    app: &AppHandle,
    stage: &mut Stage,
    processing_started: &mut Option<Instant>,
    binding_id: &str,
    hotkey_string: &str,
) {
    let Some(action) = ACTION_MAP.get(binding_id) else {
        warn!("No action in ACTION_MAP for '{binding_id}'");
        return;
    };
    let t = Instant::now();
    action.stop(app, binding_id, hotkey_string);
    if t.elapsed() > ACTION_STALL {
        warn!(
            "coordinator: action.stop blocked for {:?} — main-thread (tray/overlay) \
             or audio-device stall; hotkey presses queued during this time",
            t.elapsed()
        );
    }
    info!("coordinator: stage Processing (binding={binding_id})");
    *processing_started = Some(Instant::now());
    *stage = Stage::Processing;
    publish_stage(stage);
}

<p align="center">
  <img src=".github/assets/logo.svg" width="104" alt="">
</p>
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/assets/wordmark-dark.svg">
    <img src=".github/assets/wordmark-light.svg" width="430" alt="klava-nevinovata">
  </picture>
</p>

> Personal fork of [cjpais/Handy](https://github.com/cjpais/Handy) with Russian-language transcription tweaks.
> Pre-built installers (unsigned) on [Releases](https://github.com/egsok/klava-nevinovata/releases). Build from source for everything else.
> [English] · [Русский](README.ru.md)

This is my personal daily-driver fork of [Handy](https://github.com/cjpais/Handy), the offline speech-to-text Tauri app. I use it for Russian transcription on Windows. The fork stays close to upstream — I cherry-pick fixes and add small features that solve concrete problems I hit. Nothing here is meant to replace upstream Handy; if you don't have the same Russian-specific pain points, just use the original.

Fork is based on upstream `v0.9.3`.

## What's different from upstream

- **Two-ink design.** The fork wears its own visual identity — the two-ink print-workshop language of the [«Нейросеть не виновата»](https://t.me/neiroset_ne_vinovata) channel: magenta + deep violet inks on kraft paper (light) and ink wall (dark), IBM Plex type, a mic-with-halo mark where the inks deliberately don't register, and a recording overlay restyled as a printed sheet. Upstream's layout is untouched — only the paint.
- **Anti-hallucination defenses for Whisper models.** On silence and trailing pauses Whisper (large-v3-turbo especially) emits repetition loops and phrases memorized from its subtitle-heavy training data — "Продолжение следует...", subtitle credits, `[музыка]` tags. The fork caps the decoder's carried context (`max_prev_context_tokens=128`) and cleans the transcript afterwards: collapses runs of identical sentences and drops known hallucinated phrases on whole-sentence match only, so real speech containing the same words survives.
- **Custom transcription prompt with Russian primers.** Per-language initial prompt field in Settings → Advanced for Whisper models, shipped with primers tuned for Russian. Useful for forcing recognition of names, jargon and stylistic preferences.
- **Long-dictation punctuation fix.** whisper.cpp processes audio in 30-second windows; upstream resets decoder context between them, which drops punctuation after the first window on long dictations. This fork sets `condition_on_prev_tokens=true` so decoded context carries across windows.
- **Cyrillic word-boundary fixes for Breeze ASR.** Five regex passes unglue words that the Mandarin-trained Breeze model joins together (`cyr.cyr`, `lat.cyr`, `cyrCYR`, `latCYR`, single-letter variants, uppercase Latin acronyms). Applies to both the legacy and the GGUF-catalog Breeze models, pure post-process in `src-tauri/src/audio_toolkit/text.rs`.
- **Multi-format clipboard preservation.** Paste-and-restore preserves Files / Image / HTML / Text — not just the plain-text payload. Lets you keep a clipboard you copied earlier even if klava-nevinovata hijacks the buffer to inject the transcript.
- **Hotkey reliability on Windows.** [handy-keys fork](https://github.com/egsok/handy-keys-fork) reads live modifier state (`GetAsyncKeyState`) instead of incremental tracking — fixes transient "hotkey deafness" and phantom stuck modifiers — plus a watchdog that detects pipeline stalls.
- **Clipboard hang fix.** A hung clipboard owner can no longer freeze the main thread during paste.
- **Atomic settings updates.** All settings writes are serialized under a mutex; the upstream read-modify-write race could silently reset settings (e.g. history retention) and destroy data.

## Download

Pre-built installers are published to [Releases](https://github.com/egsok/klava-nevinovata/releases). The releases page also carries the older `0.8.3-N` stable line as a fallback.

- **Windows:** download `klava-nevinovata_0.9.3-N_x64-setup.exe` (NSIS) or `.msi` (N is the fork release number — 1, 2, ...). On first launch Windows SmartScreen will show "Windows protected your PC" — click **More info** → **Run anyway**. The binary is unsigned (see Build below).
- **Linux:** download `klava-nevinovata_..._amd64.deb` / `.AppImage` / `.rpm` for your distro.
- **macOS:** two builds — pick by your Mac's chip:
  - `klava-nevinovata_..._aarch64.dmg` — **Apple Silicon** Mac (M1 / M2 / M3 / M4, models from late 2020 onwards)
  - `klava-nevinovata_..._x64.dmg` — **Intel** Mac (older models, ~2006–2020)

  Not sure which? Click → **About This Mac**. If it lists a "Chip" like "Apple M1" → `aarch64`. If it lists a "Processor" like "Intel Core i7" → `x64`.

  Drag `klava-nevinovata.app` to `/Applications`. On first launch macOS will show **"klava-nevinovata is damaged and can't be opened, you should move it to the Bin"** — this is misleading; the app is not damaged, it's just unsigned and quarantined. Fix by removing the quarantine attribute in Terminal:

  ```bash
  xattr -d com.apple.quarantine /Applications/klava-nevinovata.app
  ```

  (If that errors with permission, try `sudo xattr -cr /Applications/klava-nevinovata.app`.) After this the app launches normally. The right-click → Open workaround that older guides mention no longer works on macOS 15+ for unsigned apps. Please report any post-launch issues in [issues](https://github.com/egsok/klava-nevinovata/issues).

## Build

If you want the bleeding edge, a platform not covered by releases, or want to audit the build yourself, build locally. (Otherwise grab a pre-built installer from [Download](#download) above.)

1. Follow upstream's [BUILD.md](BUILD.md) for platform prerequisites.
2. On Windows, set `CARGO_TARGET_DIR` to a short path (e.g. `d:/t/handy9`) — the generated Vulkan shader sources overflow MAX_PATH otherwise — and limit parallelism with `CARGO_BUILD_JOBS=8`, or MSVC runs out of heap compiling the shader-embed translation units.
3. `bun install && bun run tauri build`.

## Upstream

This fork tracks [cjpais/Handy](https://github.com/cjpais/Handy). For everything not listed above — installation, troubleshooting, platform-specific notes, model management, signal handling, CLI flags — see the [upstream README](https://github.com/cjpais/Handy/blob/main/README.md). I don't duplicate that here so it doesn't go stale relative to upstream.

If you want the official, supported app: get it from [handy.computer](https://handy.computer) or [cjpais/Handy/releases](https://github.com/cjpais/Handy/releases).

## Author

Built by [Egor Sokolov](https://egorsokolov.ru/) — 10 years in product (Sberbank, Rolf, Claustrophobia). Writing and experimenting with AI tooling — mostly Claude Code, Codex, and dev workflow tooling. I use klava-nevinovata daily for Russian voice notes; this fork is what fell out of that.

Telegram channel about AI tooling: [@neiroset_ne_vinovata](https://t.me/neiroset_ne_vinovata).

Other open experiments: [plan-tango](https://github.com/egsok/plan-tango) — a Claude ↔ Codex review loop for plans in Claude Code.

## License

MIT, inherited from cjpais/Handy — see [LICENSE](LICENSE).
Original work © cjpais and contributors. Fork changes © 2026 Egor Sokolov.

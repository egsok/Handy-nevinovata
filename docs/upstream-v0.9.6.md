# Upstream 0.9.6 integration — local 0.9.6-1

Base: fork `2bf77d5` (0.9.5-6). Merged upstream tag `v0.9.6`, commit `af48dd68a64d58aad128fdbb920492a03da53c79`.

The hotkey dependency is pinned to `egsok/handy-keys-fork` commit `2a1ae0b6d214470f3d95c4e5d71ce38f30e518ec`: upstream 0.3.4 key support plus the fork's hook priority and modifier-resynchronization telemetry. Only that dependency branch was published; this application update is local.

## Preservation decisions

- Keep the custom Whisper prompt, rolling 128-token context, RMS gate, Breeze word-boundary repair, Whisper-only repetition/hallucination cleanup and fail-open output. Upstream filler removal uses output-language evidence; disabling it does not disable the other repairs. Do not strip standalone «Спасибо».
- Adopt transcribe-cpp 0.2 and GPU schema 2. Clear legacy numeric device selections once; persist stable GPU identities. Preserve all unrelated preferences. New theme/filler/microphone mutators use the fork's atomic settings helpers.
- Keep multi-format clipboard snapshots, a one-second read timeout and asynchronous restoration. Upstream injection errors now schedule restoration too. CopyToClipboard continues to skip restoration.
- Adopt upstream's single-writer tray updates, retain fork recovery actions and slow-main-thread diagnostics.
- Keep the production identity, migration backup/non-overwrite behavior, recording retention, history defaults, updater trust configuration, installer lock wait, macOS permission recovery and NNV visual identity. Overlay remains paper-colored with the new microphone waiting state.

## Local test artifact

The portable test build uses a temporary Tauri config outside tracked source: product name `klava-nevinovata-test`, identifier `ru.egorsokolov.klava-nevinovata.test`, no updater signing artifacts. This prevents its login registration and single-instance identity from conflicting with the installed app. Autostart is disabled only in the private copied test profile. Production configuration in this repository is unchanged apart from the version.

The ZIP contains executable/resources/runtime DLLs and the portable marker, without personal data. The ready-to-run local folder additionally receives a copied settings store, SQLite snapshot and local models/recordings. Close the normal app before exercising hotkeys in the test instance.

## Validation record

Final command results, artifact SHA-256 and A/B observations are recorded alongside the local artifacts. A successful build does not establish interactive microphone/hotkey/UI behavior or macOS/Linux runtime compatibility; these remain explicit manual checks.

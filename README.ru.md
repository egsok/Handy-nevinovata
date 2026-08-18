<p align="center">
  <img src=".github/assets/logo.svg" width="104" alt="">
</p>
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/assets/wordmark-dark.svg">
    <img src=".github/assets/wordmark-light.svg" width="430" alt="klava-nevinovata">
  </picture>
</p>

> **⚠️ ЛИНИЯ АРХИВИРОВАНА.** В этой ветке живёт завершённая линия релизов 0.8.3 (последний релиз — v0.8.3-6, август 2026). Она получала фичи форка на время переезда на базу 0.9 и сохранена только как точка отката. Актуальная разработка и все свежие релизы — в ветке [`port/v0.9.0`](https://github.com/egsok/klava-nevinovata/tree/port/v0.9.0).

> Персональный форк [cjpais/Handy](https://github.com/cjpais/Handy) с правками под русскую транскрипцию.
> Готовые установщики (без подписи) — на [Releases](https://github.com/egsok/klava-nevinovata/releases). Сборка из исходников — для всего остального.
> [English](README.md) · [Русский]

Это мой персональный daily-форк [Handy](https://github.com/cjpais/Handy) — оффлайн speech-to-text приложения на Tauri. Использую для русской транскрипции под Windows. Форк держится близко к upstream: cherry-pick'и фиксов плюс небольшие фичи под конкретные проблемы, которые ловлю в работе. Замена upstream это не пытается быть — если у тебя нет таких же специфических болячек на русском, просто бери оригинал.

База: upstream `main` HEAD `10a4c31` (+5 docs-коммитов после `v0.8.3`).

## Чем отличается от upstream

- **Custom transcription prompt.** Поле под язык в Settings → Advanced для Whisper-моделей — задаёшь initial prompt. Полезно, чтобы вытаскивать имена, термины и стилистику.
- **Multi-format clipboard preservation.** Paste-and-restore теперь сохраняет ВСЕ форматы буфера (Files / Image / HTML / Text), не только plain text. Если в буфере было что-то посерьёзнее текста, и klava-nevinovata перехватил буфер для вставки транскрипта — после возврата не теряется.
- **Anti-hallucination toggle для Whisper.** Переключатель в Settings → Advanced применяет `n_max_text_ctx=128` + `entropy_thold=2.8` к `WhisperInferenceParams` (эквивалент OpenWhispr PR #552 / whisper.cpp#1507). Гасит зацикленные галлюцинации на длинных тишинах ("и я хочу и я хочу...").
- **Cyrillic word-boundary фиксы для Breeze ASR.** Пять regex-проходов разлепляют слова, которые Mandarin-trained Breeze склеивает (`cyr.cyr`, `lat.cyr`, `cyrCYR`, `latCYR`, single-letter варианты, uppercase Latin аббревиатуры). Гейтится `selected_model == "breeze-asr"`, чистый post-process в `src-tauri/src/audio_toolkit/text.rs`.
- **`/O2` фикс компилятора в whisper-rs-sys форке.** cmake 4.2.3 + VS2022 молча дропает `CMAKE_*_FLAGS_RELEASE` инициализаторы → whisper.cpp собирается с `/Od` вместо `/O2`. Форк явно задаёт `/MD /O2 /Ob2 /DNDEBUG`. Эмпирически проверено: `large-v3` RTF 0.10 → 0.06 (~1.67× быстрее Whisper-инференса на Windows).

Первые четыре — коммиты поверх upstream в этом репозитории. `/O2` фикс лежит в трёх соседних форках Rust-стека whisper (`whisper-rs-sys-fork`, `whisper-rs-fork`, `transcribe-rs-fork`, все на ветке `daily-stable`), подключаются через `[patch.crates-io]` пути в `src-tauri/Cargo.toml`. Если убрать эти patch'и — отвалятся и `/O2`, и backend для anti-hallucination.

## Загрузка

Готовые установщики публикуются в [Releases](https://github.com/egsok/klava-nevinovata/releases).

- **Windows:** скачай `klava-nevinovata_0.8.3-N_x64-setup.exe` (NSIS) или `.msi` (N — номер форк-релиза: 1, 2, ...). При первом запуске Windows SmartScreen покажет "Windows protected your PC" — кликни **More info** → **Run anyway**. Бинарь не подписан (см. Сборка ниже).
- **Linux:** скачай `klava-nevinovata_..._amd64.deb` / `.AppImage` / `.rpm` под свой дистрибутив.
- **macOS:** две сборки — выбирай по чипу твоего Mac'а:
  - `klava-nevinovata_..._aarch64.dmg` — **Apple Silicon** Mac (M1 / M2 / M3 / M4, модели с конца 2020 года и новее)
  - `klava-nevinovata_..._x64.dmg` — **Intel** Mac (старые модели, ~2006–2020)

  Не уверен какой у тебя? Клик по → **About This Mac** (или «Об этом Mac»). Если строка "Chip" / «Чип» с надписью "Apple M1" и т.п. → нужен `aarch64`. Если строка "Processor" / «Процессор» с "Intel Core ..." → нужен `x64`.

  Перетащи `klava-nevinovata.app` в `/Applications`. При первом запуске macOS покажет **"klava-nevinovata is damaged and can't be opened, you should move it to the Bin"** — это вводящее в заблуждение сообщение; приложение не повреждено, оно просто не подписано и помечено quarantine-атрибутом при скачивании. Фикс — удалить quarantine через Терминал:

  ```bash
  xattr -d com.apple.quarantine /Applications/klava-nevinovata.app
  ```

  (Если ругнётся на permissions, попробуй `sudo xattr -cr /Applications/klava-nevinovata.app`.) После этого приложение запускается нормально. Старый workaround "right-click → Open" на macOS 15+ для неподписанных приложений больше не работает. Если после запуска не выдаётся разрешение «Универсальный доступ» — смотри [Решение проблем](#решение-проблем). Баги после запуска репортить в [issues](https://github.com/egsok/klava-nevinovata/issues).

## Решение проблем

**macOS: галочка «Универсальный доступ» не сохраняется / приложение снова и снова просит доступ.** macOS привязывает это разрешение к «личности» приложения, и устаревшая запись — от оригинального Handy или от старой версии этого приложения — блокирует новую. Что делать:

1. Открой **Системные настройки → Конфиденциальность и безопасность → Универсальный доступ** и удали записи Handy / klava-nevinovata кнопкой **−**.
2. Если оригинальный Handy (или старая версия этого приложения) всё ещё лежит в `/Applications` и ты им не пользуешься — удали его.
3. Сбрось сохранённое разрешение в Терминале (кнопка «Сбросить разрешение» на экране онбординга выполняет первую команду за тебя):

   ```bash
   tccutil reset Accessibility ru.egorsokolov.klava-nevinovata
   ```

   Если когда-то стоял оригинальный Handy или любой более ранний релиз klava-nevinovata (они использовали старый идентификатор) — дополнительно выполни `tccutil reset Accessibility com.pais.handy` (учти: это сбросит разрешение и у оригинального Handy, если ты им пользуешься).

4. Перезапусти приложение и выдай разрешение заново.

**macOS: после обновления приложение снова просит разрешения.** Пока это ожидаемо: сборки не подписаны сертификатом Apple Developer, и каждый обновлённый бинарник macOS считает новым приложением. Выдай разрешение заново и работай дальше.

## Сборка

Если хочешь bleeding edge, платформу не покрытую релизами или сам проверить билд — собирай локально. (Иначе бери готовый установщик из [Загрузка](#загрузка) выше.)

1. Платформенные пререквизиты — в upstream [BUILD.md](BUILD.md).
2. Три соседних форка должны лежать рядом с `klava-nevinovata/` и быть на ветке `daily-stable` (см. секцию `[patch.crates-io]` в `src-tauri/Cargo.toml`).
3. На Windows ставь `CARGO_TARGET_DIR=d:/t/handy` (короткий путь — Vulkan-шейдеры whisper-rs-sys иначе упираются в MAX_PATH) и `CARGO_BUILD_JOBS=2` (release-профиль с `lto=true`, параллельный линк OOM'нется на 16 ГБ).
4. `bun install && bun run tauri build`. Финальное "failed to bundle project: program not found" игнорируй — это custom-signing step, сам `.exe` собран нормально.

## Upstream

Форк трекает [cjpais/Handy](https://github.com/cjpais/Handy). Всё, что не упомянуто выше — установка, troubleshooting, платформенные заметки, управление моделями, signal handling, CLI-флаги — смотри в [upstream README](https://github.com/cjpais/Handy/blob/main/README.md). Дублировать не стал, чтобы не разъезжалось с актуальной версией.

Если нужен официальный поддерживаемый Handy — забирай с [handy.computer](https://handy.computer) или со страницы релизов [cjpais/Handy/releases](https://github.com/cjpais/Handy/releases).

## Автор

Сделал [Егор Соколов](https://egorsokolov.ru/) — 10 лет в продукте (Сбер, Рольф, Клаустрофобия). Пишу и экспериментирую с AI-инструментами — в основном Claude Code, Codex и dev-воркфлоу. Сам пользуюсь klava-nevinovata для русских голосовых заметок; этот форк — то, что из этого выпало в код.

Telegram-канал про AI-инструменты: [@neiroset_ne_vinovata](https://t.me/neiroset_ne_vinovata).

Другие открытые эксперименты: [plan-tango](https://github.com/egsok/plan-tango) — Claude ↔ Codex ревью-цикл для планов в Claude Code.

## Лицензия

MIT, наследуется от cjpais/Handy — см. [LICENSE](LICENSE).
Исходная работа © cjpais и контрибьюторы. Правки форка © 2026 Егор Соколов.

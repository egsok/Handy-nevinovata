<p align="center">
  <img src=".github/assets/logo.svg" width="104" alt="">
</p>
<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset=".github/assets/wordmark-dark.svg">
    <img src=".github/assets/wordmark-light.svg" width="430" alt="klava-nevinovata">
  </picture>
</p>

> Персональный форк [cjpais/Handy](https://github.com/cjpais/Handy) с правками под русскую транскрипцию.
> Готовые установщики (без подписи) — на [Releases](https://github.com/egsok/klava-nevinovata/releases). Сборка из исходников — для всего остального.
> [English](README.md) · [Русский]

Это мой персональный daily-форк [Handy](https://github.com/cjpais/Handy) — оффлайн speech-to-text приложения на Tauri. Использую для русской транскрипции под Windows. Форк держится близко к upstream: cherry-pick'и фиксов плюс небольшие фичи под конкретные проблемы, которые ловлю в работе. Замена upstream это не пытается быть — если у тебя нет таких же специфических болячек на русском, просто бери оригинал.

База: upstream `v0.9.3`.

## Чем отличается от upstream

- **Дизайн в две краски.** У форка собственный визуальный язык — двухкрасочная «печатная мастерская» канала [«Нейросеть не виновата»](https://t.me/neiroset_ne_vinovata): magenta + глубокий фиолет на крафт-бумаге (светлая тема) и чернильной стене (тёмная), шрифты IBM Plex, знак «микрофон с нимбом», где краски нарочно не сведены, и оверлей записи в виде печатного листа. Раскладка upstream не тронута — только краска.
- **Защита от галлюцинаций Whisper-моделей.** На тишине и паузах в хвосте Whisper (особенно large-v3-turbo) выдаёт зацикленные повторы и фразы, заученные из субтитров: «Продолжение следует...», титры субтитров, теги `[музыка]`. Форк ограничивает переносимый контекст декодера (`max_prev_context_tokens=128`) и чистит транскрипт после: схлопывает серии одинаковых предложений и убирает известные фразы-галлюцинации — только по полному совпадению предложения, так что настоящая речь с теми же словами не страдает.
- **Custom transcription prompt с русскими primer'ами.** Поле под язык в Settings → Advanced для Whisper-моделей — задаёшь initial prompt; из коробки идут primer'ы, настроенные под русский. Полезно, чтобы вытаскивать имена, термины и стилистику.
- **Фикс пунктуации на длинных диктовках.** whisper.cpp обрабатывает аудио 30-секундными окнами; upstream сбрасывает контекст декодера между ними, из-за чего после первого окна пропадает пунктуация. Форк ставит `condition_on_prev_tokens=true` — контекст переносится между окнами.
- **Cyrillic word-boundary фиксы для Breeze ASR.** Пять regex-проходов разлепляют слова, которые Mandarin-trained Breeze склеивает (`cyr.cyr`, `lat.cyr`, `cyrCYR`, `latCYR`, single-letter варианты, uppercase Latin аббревиатуры). Работает и для легаси-модели, и для GGUF-каталожной Breeze; чистый post-process в `src-tauri/src/audio_toolkit/text.rs`.
- **Multi-format clipboard preservation.** Paste-and-restore сохраняет ВСЕ форматы буфера (Files / Image / HTML / Text), не только plain text. Если в буфере было что-то посерьёзнее текста, и klava-nevinovata перехватил буфер для вставки транскрипта — после возврата не теряется.
- **Надёжность хоткеев на Windows.** [Форк handy-keys](https://github.com/egsok/handy-keys-fork) читает живое состояние модификаторов (`GetAsyncKeyState`) вместо инкрементального трекинга — лечит транзиентную «глухоту» хоткея и фантомные залипшие модификаторы; плюс watchdog, детектящий зависание пайплайна.
- **Фикс зависания на буфере обмена.** Зависший владелец clipboard'а больше не может заморозить главный поток при вставке.
- **Атомарные обновления настроек.** Все записи настроек сериализованы под мьютексом; upstream'овская read-modify-write гонка могла молча сбросить настройки (например, retention истории) и уничтожить данные.

## Загрузка

Готовые установщики публикуются в [Releases](https://github.com/egsok/klava-nevinovata/releases). Там же лежит старшая стабильная линия `0.8.3-N` — как запасной вариант.

- **Windows:** скачай `klava-nevinovata_0.9.3-N_x64-setup.exe` (NSIS) или `.msi` (N — номер форк-релиза: 1, 2, ...). При первом запуске Windows SmartScreen покажет "Windows protected your PC" — кликни **More info** → **Run anyway**. Бинарь не подписан (см. Сборка ниже).
- **Linux:** скачай `klava-nevinovata_..._amd64.deb` / `.AppImage` / `.rpm` под свой дистрибутив.
- **macOS:** две сборки — выбирай по чипу твоего Mac'а:
  - `klava-nevinovata_..._aarch64.dmg` — **Apple Silicon** Mac (M1 / M2 / M3 / M4, модели с конца 2020 года и новее)
  - `klava-nevinovata_..._x64.dmg` — **Intel** Mac (старые модели, ~2006–2020)

  Не уверен какой у тебя? Клик по → **About This Mac** (или «Об этом Mac»). Если строка "Chip" / «Чип» с надписью "Apple M1" и т.п. → нужен `aarch64`. Если строка "Processor" / «Процессор» с "Intel Core ..." → нужен `x64`.

  Перетащи `klava-nevinovata.app` в `/Applications`. При первом запуске macOS покажет **"klava-nevinovata is damaged and can't be opened, you should move it to the Bin"** — это вводящее в заблуждение сообщение; приложение не повреждено, оно просто не подписано и помечено quarantine-атрибутом при скачивании. Фикс — удалить quarantine через Терминал:

  ```bash
  xattr -d com.apple.quarantine /Applications/klava-nevinovata.app
  ```

  (Если ругнётся на permissions, попробуй `sudo xattr -cr /Applications/klava-nevinovata.app`.) После этого приложение запускается нормально. Старый workaround "right-click → Open" на macOS 15+ для неподписанных приложений больше не работает. Баги после запуска репортить в [issues](https://github.com/egsok/klava-nevinovata/issues).

  **Разрешения на macOS.** При первом запуске приложение просит два разрешения: микрофон (слышать тебя) и Accessibility / Универсальный доступ (впечатывать расшифровку в другие приложения). Экран разрешений показывается только при запуске — если закрыл его, перезапусти приложение или выдай доступ руками: **Настройки системы → Конфиденциальность и безопасность → Универсальный доступ** → включи klava-nevinovata (микрофон — в том же списке «Конфиденциальность и безопасность»). Если тумблер Универсального доступа уже включён, а приложение всё равно пишет «Ожидание»: выбери klava-nevinovata в этом списке, удали кнопкой **−** и добавь заново через **+** — после обновления неподписанного приложения macOS иногда держит разрешение за старой версией. Перезапуск приложения после выдачи тоже помогает.

## Сборка

Если хочешь bleeding edge, платформу не покрытую релизами или сам проверить билд — собирай локально. (Иначе бери готовый установщик из [Загрузка](#загрузка) выше.)

1. Платформенные пререквизиты — в upstream [BUILD.md](BUILD.md).
2. На Windows ставь `CARGO_TARGET_DIR` в короткий путь (например, `d:/t/handy9`) — сгенерированные исходники Vulkan-шейдеров иначе упираются в MAX_PATH — и ограничь параллелизм `CARGO_BUILD_JOBS=8`, иначе MSVC ловит out of heap space на shader-embed файлах.
3. `bun install && bun run tauri build`.

## Upstream

Форк трекает [cjpais/Handy](https://github.com/cjpais/Handy). Всё, что не упомянуто выше — установка, troubleshooting, платформенные заметки, управление моделями, signal handling, CLI-флаги — смотри в [upstream README](https://github.com/cjpais/Handy/blob/main/README.md). Дублировать не стал, чтобы не разъезжалось с актуальной версией.

Если нужен официальный поддерживаемый Handy — забирай с [handy.computer](https://handy.computer) или со страницы релизов [cjpais/Handy/releases](https://github.com/cjpais/Handy/releases).

## Автор

Сделал [Егор Соколов](https://egorsokolov.ru/) — 10 лет в продукте (Сбер, Рольф, Клаустрофобия). Пишу и экспериментирую с AI-инструментами — в основном Claude Code, Codex и dev-воркфлоу. Сам пользуюсь klava-nevinovata для русских голосовых заметок; этот форк — то, что из этого выпало в код.

Telegram-канал про AI-инструменты: [@neiroset_ne_vinovata](https://t.me/neiroset_ne_vinovata).

Другие открытые эксперименты:

- [plan-tango](https://github.com/egsok/plan-tango) — Claude ↔ Codex ревью-цикл для планов в Claude Code.
- [press-1](https://github.com/egsok/press-1) — отвечать на permission-промпты Claude Code одной клавишей из любого окна.
- [napotom](https://github.com/egsok/napotom) — десктопный загрузчик видео с очередью, дружелюбная обёртка над yt-dlp.

## Лицензия

MIT, наследуется от cjpais/Handy — см. [LICENSE](LICENSE).
Исходная работа © cjpais и контрибьюторы. Правки форка © 2026 Егор Соколов.

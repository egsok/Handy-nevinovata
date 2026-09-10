# «Клава не виновата» 0.9.6-1

Обновление на базе Handy 0.9.6 с сохранением оформления и доработок «Клавы»: пользовательского промпта, контекста длинной диктовки и исправлений текста Whisper/Breeze.

## Напоминание: обновиться можно прямо из приложения

**Обновление из приложения доступно начиная с версии 0.9.5-3. Нажмите «Проверить обновления» внизу окна, затем «Доступно обновление».** «Клава» скачает и установит новую версию, затем перезапустится. Проверка может выполняться при запуске; загрузка и установка начинаются только после вашего нажатия. Если кнопки нет, включите проверку обновлений в настройках.

**Portable-версия обновляется вручную:** кнопка откроет окно со ссылкой на загрузку. Закройте приложение и обновите его в той же папке, сохранив `Data/` с настройками, моделями и записями.

## Что изменилось

- **Удаление слов-паразитов можно отключить** в расширенных настройках. При включённом удалении учитывается язык распознанного текста, а не язык интерфейса; исправления повторов и склеенных слов продолжают работать независимо от переключателя.
- **В пользовательский словарь можно добавлять фразы из нескольких слов**, например «Клава не виновата» или «Visual Studio Code».
- **Индикатор записи показывает ожидание микрофона.** Сигнал готовности появляется, когда микрофон начинает передавать звук.
- **После отключения выбранного микрофона приложение переходит на системный**, если он доступен; при успешном подключении выбор обновляется и в настройках.
- **Выбор конкретной видеокарты сохраняется между запусками.** При переходе с 0.9.5 ранее выбранная видеокарта сбрасывается в Auto — при необходимости выберите её заново.
- **Движок transcribe.cpp обновлён до 0.2.0.** Исправлены конфликты обновления значка в трее; в portable-режиме новые загрузки Hugging Face хранятся внутри `Data/`.

---

# Klava ne vinovata 0.9.6-1 — English

Based on Handy 0.9.6, with Klava's design and custom features preserved: transcription prompts, context across long dictations, and Whisper/Breeze text cleanup.

## Reminder: update from inside the app

**In-app updates are available in installed copies starting with version 0.9.5-3. Click “Check for updates” at the bottom of the window, then “Update available”.** Klava downloads and installs the update, then restarts. It may check for updates at startup; downloading and installation begin only after your click. If the button is missing, enable update checks in settings.

**Portable copies require a manual update:** the button opens a dialog with a download link. Close the app and update it in the same folder, keeping `Data/` with your settings, models, and recordings.

## What's changed

- **Filler-word removal can be turned off** in Advanced Settings. When enabled, it uses evidence about the transcription's language instead of the interface language; repetition and word-boundary cleanup remain active independently.
- **Custom vocabulary accepts multiword phrases**, such as “Klava ne vinovata” or “Visual Studio Code”.
- **The recording overlay shows when the microphone is still starting.** The ready indication appears once audio begins arriving.
- **Disconnecting the selected microphone triggers a fallback to the system default**, if available; a successful fallback also updates the selection in settings.
- **A specific GPU selection persists across restarts.** Upgrading from 0.9.5 resets an explicit GPU choice to Auto; select your GPU again if needed.
- **transcribe.cpp has been updated to 0.2.0.** Tray icon update conflicts are fixed, and new Hugging Face downloads stay inside `Data/` in portable mode.

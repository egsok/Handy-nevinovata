# v0.9.5-5 — хвосты аудита: миграция, апдейтер, онбординг

> Черновик для описания GitHub-релиза. English version below.

## Главное

Чистый фикс-релиз: добиты хвосты аудита v0.9.5-4. Ничего нового в интерфейсе.

## Что исправлено

**Миграция данных больше не может замедлять каждый запуск.** Если перенос данных из старой папки Handy стабильно срывался (например, антивирус держит базу истории), приложение повторяло полный прогон при каждом старте — в худшем случае это добавляло до ~10 секунд до появления окна. Теперь попыток три: после третьей неудачи миграция честно сдаётся, пишет путь к старым данным в лог и больше не мешает запуску. Свежая установка без старых данных тоже перестала проверять их наличие при каждом старте.

**Апдейтер.** У пользователей с выключенной проверкой обновлений кнопка проверки больше не мигает в футере на мгновение при запуске. Проверка обновлений из трея больше не может запуститься параллельно с уже идущей установкой.

**Онбординг на macOS.** Закрыта лазейка вокруг сброса разрешения: после успешного сброса кнопка «Предоставить разрешение» остаётся заблокированной, даже если повторный сброс не удался, а панель с кнопкой перезапуска не даёт себя спрятать, пока перезапуск действительно нужен. Главное — кнопка выдачи разрешения микрофона больше не может «протащить» онбординг мимо этой блокировки: устаревшее разрешение Accessibility, которое macOS показывает до перезапуска, больше не засчитывается.

---

# v0.9.5-5 — audit follow-ups: migration, updater, onboarding (English)

A pure fix release finishing the follow-ups from the v0.9.5-4 audit. Nothing new in the UI.

**Data migration can no longer slow down every launch:** if migrating data from the old Handy folder kept failing (e.g. an antivirus holding the history database), the full run was repeated on every startup — up to ~10 seconds before the window appeared. It now gives up after three failed attempts, logs where the legacy data lives, and stops interfering. Fresh installs without legacy data no longer probe for it on every launch either.

**Updater:** users with update checks disabled no longer see the check button flash in the footer for a moment on launch; a tray-triggered check can no longer run in parallel with an ongoing install.

**macOS onboarding:** the permission-reset flow is sealed — after a successful reset the Grant button stays locked even if a later re-reset fails, the panel with the Restart button can't be hidden while a restart is actually needed, and most importantly the microphone Grant button can no longer sneak onboarding past the lock by trusting the stale Accessibility grant macOS reports until the restart.

# v0.9.5-4 — надёжность после аудита: миграция, апдейтер, онбординг

> Черновик для описания GitHub-релиза. English version below.

## Главное

Это релиз надёжности по итогам подробного аудита v0.9.5-3. Ничего нового в интерфейсе — зато несколько сценариев, где данные или обновления могли тихо потеряться, теперь ведут себя честно. И это первый релиз, который прилетит пользователям v0.9.5-3 **автоматически** — через встроенный апдейтер.

## Что исправлено

**Миграция данных из старой папки.** Если при первом запуске антивирус держал базу истории — раньше миграция молча сдавалась, и история с настройками терялись навсегда. Теперь неудачная миграция честно повторяется при следующем запуске: перенос истории и записей догоняется, а настройки, которые вы успели изменить, никогда не перезаписываются. Необратимо повреждённая база больше не блокирует перенос всего остального.

**Апдейтер.** Ошибки проверки и установки обновлений больше не молчат — показывается понятное сообщение. Длинные надписи («Проверка обновлений…») больше не ломают вёрстку футера в русском интерфейсе. Выключенная проверка обновлений теперь означает тишину, а не постоянную надпись «Проверка обновлений отключена».

**Онбординг на macOS.** Дожаты хвосты сценария сброса разрешения: после сброса кнопка «Предоставить разрешение» блокируется до перезапуска (иначе macOS могла «зачесть» устаревшее разрешение), инструкция про удаление старых записей Handy остаётся видимой, а неудачный повторный сброс больше не выглядит успешным.

**Безопасность релизного конвейера.** Ключ подписи обновлений убран из пути тестовых PR-сборок — код из чужого pull request'а теперь физически не может до него дотянуться.

## Линия 0.8.3 закрыта

Ветка 0.8.3 (страховочная на время переезда на базу 0.9) архивирована: релизов из неё больше не будет, последний — v0.8.3-6. Все её релизы помечены как Pre-release и остаются доступными для скачивания.

---

# v0.9.5-4 — post-audit reliability: migration, updater, onboarding (English)

A reliability release following a deep audit of v0.9.5-3. Nothing new in the UI — but several scenarios where data or updates could quietly get lost now behave honestly. This is also the first release that v0.9.5-3 users receive **automatically** via the in-app updater.

**Data migration:** if an antivirus held the history database on first launch, the migration used to give up silently, losing history and settings forever. A failed migration now genuinely retries on the next launch — history and recordings catch up, and settings you've changed since are never overwritten. A permanently damaged database no longer blocks migrating everything else.

**Updater:** check/install failures now show a clear message instead of dying in the console; long labels no longer break the footer layout in Russian; disabling update checks now means silence, not a permanent "checking disabled" label.

**macOS onboarding:** after a permission reset the Grant button is disabled until the restart the flow asks for (macOS could otherwise accept a stale grant), the stale-entries recipe stays visible, and a failed re-reset no longer looks successful.

**Release pipeline security:** the update-signing key is no longer reachable from PR test builds.

**The 0.8.3 line is archived:** no further releases from it; the last one is v0.8.3-6, and all its releases are marked Pre-release but remain downloadable.

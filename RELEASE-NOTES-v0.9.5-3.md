# v0.9.5-3 — встроенные обновления и закалённая миграция данных

> Черновик для описания GitHub-релиза. English version below.

## Главное

Приложение снова умеет обновляться само. Проверка обновлений живёт в футере окна настроек и в меню трея («Проверить обновления»); скачивание и установка — в один клик, обновления подписаны собственным ключом форка и тянутся из GitHub-релизов этого репозитория.

Важно: **это первый релиз со встроенными обновлениями**. Тем, кто сидит на v0.9.5-2 и старше, этот релиз нужно поставить вручную — зато следующие уже приедут сами.

## Что ещё изменилось

**Миграция данных стала бережнее.** Модели и аудиозаписи из старой папки `com.pais.handy` теперь переносятся жёсткими ссылками: мгновенно, без дублирования места на диске, и оригинальный Handy (если он установлен) сохраняет все свои файлы — при этом «Клаве» больше не приходится перекачивать модели. Дополнительно: история больше не бросается молча, если файл базы оказался временно заблокирован антивирусом; копия истории проверяется на целостность перед использованием; миграция не может зависнуть на старте, даже если старое приложение продолжает писать в базу.

**Онбординг на macOS.** После сброса разрешения «Универсальный доступ» приложение теперь прямо ведёт к перезапуску (это единственный надёжный следующий шаг), а не обратно к кнопке «Предоставить разрешение», которая могла завершить онбординг по устаревшему состоянию системы.

## macOS: известное ограничение

Пока сборки не подписаны сертификатом Apple Developer, после каждого обновления (в том числе встроенного) разрешение «Универсальный доступ» нужно выдавать заново. Кнопки «Сбросить разрешение» и «Перезапустить приложение» на экране онбординга помогают, если галочка не сохраняется.

## Изменения

- **feat(updater):** встроенные обновления из GitHub-релизов форка — футер настроек, пункт в трее, подпись артефактов собственным ключом.
- **fix(migration):** hard-link вместо переноса файлов; честная обработка заблокированной базы истории; проверка целостности копии; ограничение по времени на снапшот базы.
- **fix(onboarding):** после сброса TCC-разрешения сценарий ведёт к перезапуску приложения (все 24 языка); карточка микрофона больше не зависает на спиннере после сброса.

---

# v0.9.5-3 — in-app updates, hardened data migration (English)

The app can update itself again. The update checker lives in the settings-window footer and in the tray menu ("Check for updates"); download-and-install is one click, updates are signed with the fork's own key and pulled from this repository's GitHub releases.

Note: **this is the first release with in-app updates.** If you're on v0.9.5-2 or older, install this one manually — subsequent releases will arrive on their own.

**Data migration is gentler:** models and recordings from the legacy `com.pais.handy` folder are now hard-linked — instant, no duplicated disk space, and an installed original Handy keeps every file while klava no longer re-downloads models. History migration no longer silently gives up on a temporarily locked database file, verifies the copied database's integrity before using it, and can't hang startup even if the legacy app is still writing.

**macOS onboarding:** after resetting the Accessibility permission the flow now steers you to restart the app (the only reliable next step) instead of back to Grant Permission.

**Known macOS limitation:** until builds are signed with an Apple Developer certificate, the Accessibility permission must be re-granted after every update, including in-app ones. The onboarding "Having trouble granting access?" block (permission reset + app restart) helps when the checkbox won't stick.

# v0.8.3-6 — новая идентичность приложения и починка «Универсального доступа» на macOS

> Черновик для описания GitHub-релиза. English version below.

## Главное

Приложение сменило системный идентификатор: `com.pais.handy` → `ru.egorsokolov.klava-nevinovata`. Старый идентификатор достался от оригинального Handy, и из-за этого macOS путала два приложения: если Handy был (или когда-то был) установлен, галочка «Универсальный доступ» для «Клавы» не сохранялась, и онбординг застревал. Теперь конфликт устранён навсегда — оба приложения могут жить на одном Mac.

## Что увидят пользователи при обновлении

**Все платформы.** При первом запуске новая версия автоматически перенесёт данные из старой папки: настройки, историю, аудиозаписи, кастомные звуки и скачанные модели. Старая папка (`com.pais.handy`) не удаляется — в ней остаются копии настроек и истории, удалить её можно вручную позже. Если на компьютере установлен оригинальный Handy, модели и записи не переносятся (они нужны ему) — «Клава» скачает модели заново.

**macOS.** Разрешения «Универсальный доступ» и «Микрофон» придётся выдать один раз заново — для системы приложение теперь новое. Это ожидаемо. Если разрешение не выдаётся:

1. Открой **Системные настройки → Конфиденциальность и безопасность → Универсальный доступ** и удали старые записи Handy / klava-nevinovata кнопкой **−**.
2. Нажми «Не получается выдать доступ?» на экране онбординга — там есть кнопки **«Сбросить разрешение»** и **«Перезапустить приложение»**.
3. Вручную то же самое: `tccutil reset Accessibility ru.egorsokolov.klava-nevinovata` в Терминале (если стоял оригинальный Handy или любой более ранний релиз — дополнительно `tccutil reset Accessibility com.pais.handy`).

Пока сборки не подписаны сертификатом Apple Developer, разрешение нужно будет выдавать заново после каждого обновления приложения.

**Windows.** Обновление ставится поверх как обычно. Издатель в «Установке и удалении программ» теперь «Egor Sokolov» (раньше отображался «pais»). Кастомная папка установки и обновление поверх старых `.exe`/`.msi`-установок отрабатывают штатно.

**Linux.** Папка данных переехала в `~/.local/share/ru.egorsokolov.klava-nevinovata`, перенос автоматический.

## Изменения

- **fix(macOS):** конфликт TCC-записи с оригинальным Handy — идентификатор приложения теперь собственный.
- **feat(onboarding):** блок «Не получается выдать доступ?» на шаге Accessibility — сброс TCC-записи в один клик и перезапуск приложения (все 20 языков).
- **feat(migration):** одноразовый автоматический перенос данных из старой папки; история копируется консистентным SQLite-снапшотом; старые данные сохраняются как бэкап.
- **fix(installer):** Windows-инсталлятор корректно обновляет установки прежних версий (включая MSI и кастомные папки) после смены издателя.
- **docs:** секция Troubleshooting в README (EN и RU).

---

# v0.8.3-6 — new app identity, macOS Accessibility fix (English)

The app's bundle identifier changed from `com.pais.handy` (inherited from the original Handy) to `ru.egorsokolov.klava-nevinovata`. This permanently fixes the macOS issue where the Accessibility checkbox wouldn't stick when the original Handy was (or had been) installed — macOS was conflating the two apps. Both apps can now coexist.

**On first launch the new version migrates your data** (settings, history, recordings, custom sounds, downloaded models) from the old folder automatically. The old `com.pais.handy` folder is not deleted — it keeps copies of your settings and history and can be removed manually later. If the original Handy is installed, models and recordings are left in place for it; models are re-downloaded.

**macOS:** you'll need to re-grant Accessibility and Microphone once — the app is a new identity to the system. If the permission won't stick, use the new "Having trouble granting access?" block on the onboarding screen (one-click permission reset + app restart), or run `tccutil reset Accessibility ru.egorsokolov.klava-nevinovata` in Terminal. Until builds are signed with an Apple Developer certificate, permissions must be re-granted after each update.

**Windows:** upgrades install over previous versions as usual; the Publisher shown in Add/Remove Programs is now "Egor Sokolov". **Linux:** the data dir moved to `~/.local/share/ru.egorsokolov.klava-nevinovata`; migration is automatic.

// Single source of truth for brand strings used from code.
// The product name shown in native chrome (window title, tray, installer)
// comes from tauri.conf.json's productName; keep these in sync with it.
export const APP_NAME = "klava-nevinovata";
export const REPO_URL = "https://github.com/egsok/klava-nevinovata";
export const TELEGRAM_CHANNEL_NAME = "Нейросеть не виновата";
// Separate invite links per surface so joins can be attributed to where
// the user clicked. Revoking an invite link kills it in shipped builds.
export const TELEGRAM_URL_ABOUT = "https://t.me/+42RakjtPYpxkYmNi";
export const TELEGRAM_URL_FOOTER = "https://t.me/+ds3F3vTiA10zNTky";

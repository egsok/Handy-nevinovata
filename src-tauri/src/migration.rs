//! One-time migration of user data from the legacy `com.pais.handy` app data
//! directory into the current identifier's directory.
//!
//! The legacy identifier was shared with the original upstream Handy app, so
//! the legacy directory may still be in use by it: settings and history are
//! copied, models and recordings are moved only when upstream Handy is not
//! installed, and the legacy directory is never deleted. Nothing already
//! present in the new directory is ever overwritten, so an accidental re-run
//! (e.g. after the user deletes their settings file) cannot clobber data the
//! new app has written since.

use std::fs;
use std::path::Path;

const LEGACY_IDENTIFIER: &str = "com.pais.handy";
const SETTINGS_FILE: &str = "settings_store.json";

/// Best-effort migration; never fails and never blocks startup on errors.
/// Must run before the settings store, models dir or history db are first
/// opened.
pub fn migrate_legacy_data(app: &tauri::AppHandle) {
    if crate::portable::is_portable() {
        // Portable data lives next to the exe, independent of the identifier
        return;
    }

    let new_dir = match crate::portable::app_data_dir(app) {
        Ok(dir) => dir,
        Err(e) => {
            log::warn!("migration: cannot resolve app data dir: {}", e);
            return;
        }
    };

    // The settings file doubles as the "already migrated / already in use"
    // marker. Checking directory existence would misfire: on Linux the log
    // plugin creates <new_dir>/logs before setup() runs.
    if new_dir.join(SETTINGS_FILE).exists() {
        return;
    }

    let old_dir = match new_dir.parent() {
        Some(parent) => parent.join(LEGACY_IDENTIFIER),
        None => {
            log::warn!("migration: app data dir has no parent");
            return;
        }
    };
    if !old_dir.is_dir() {
        log::info!("migration: no legacy data at {}", old_dir.display());
        return;
    }

    migrate_between(&old_dir, &new_dir, original_handy_installed());
}

/// Path-based core of the migration, separated from AppHandle resolution so
/// the whole flow is unit-testable against temp directories.
fn migrate_between(old_dir: &Path, new_dir: &Path, handy_installed: bool) {
    log::info!(
        "migration: migrating legacy data from {} to {}",
        old_dir.display(),
        new_dir.display()
    );
    if let Err(e) = fs::create_dir_all(new_dir) {
        log::error!("migration: cannot create {}: {}", new_dir.display(), e);
        return;
    }

    let ok_history = copy_history_db(old_dir, new_dir);

    // Recordings follow the same rule as models: moved (instant same-volume
    // rename, no unbounded copy at startup) unless upstream Handy still needs
    // them; its history references these files.
    let old_recordings = old_dir.join("recordings");
    let ok_recordings = if !old_recordings.is_dir() {
        true
    } else if handy_installed {
        copy_dir_recursive(&old_recordings, &new_dir.join("recordings"), 0)
    } else {
        move_dir_entries(&old_recordings, &new_dir.join("recordings"), &[])
    };

    for sound in ["custom_start.wav", "custom_stop.wav"] {
        copy_file_best_effort(&old_dir.join(sound), &new_dir.join(sound));
    }

    let old_models = old_dir.join("models");
    if old_models.is_dir() {
        if handy_installed {
            log::info!(
                "migration: original Handy detected; leaving models in place, they will be re-downloaded"
            );
        } else {
            move_dir_entries(
                &old_models,
                &new_dir.join("models"),
                &[".partial", ".extracting"],
            );
        }
    }

    // Settings are copied last: the file is the skip-marker, so it only lands
    // when the important items above made it across.
    let old_settings = old_dir.join(SETTINGS_FILE);
    if !old_settings.is_file() {
        log::info!("migration: legacy dir has no {}, done", SETTINGS_FILE);
        return;
    }
    if ok_history && ok_recordings {
        // Copy via temp file + rename so an interrupted copy cannot leave a
        // partial settings file behind (its mere existence is the skip-marker)
        let tmp = new_dir.join("settings_store.json.migrating");
        let result = fs::copy(&old_settings, &tmp)
            .and_then(|_| fs::rename(&tmp, new_dir.join(SETTINGS_FILE)));
        match result {
            Ok(_) => log::info!(
                "migration: complete; legacy settings and history kept at {}",
                old_dir.display()
            ),
            Err(e) => {
                let _ = fs::remove_file(&tmp);
                log::error!(
                    "migration: failed to copy settings: {}; legacy data remains at {}",
                    e,
                    old_dir.display()
                );
            }
        }
    } else {
        log::error!(
            "migration: important items failed; settings not copied, defaults will apply. Legacy data remains at {}",
            old_dir.display()
        );
    }
}

/// Copy the history database. Prefers SQLite's backup API — a consistent
/// snapshot even if the legacy app is writing concurrently — falling back to
/// a plain file copy plus the WAL sidecar (`-shm` is transient and skipped).
/// Databases from newer builds than this one, or files that are not SQLite
/// databases, are skipped so an unreadable copy cannot brick startup.
/// Returns `false` only when a usable source exists and neither approach
/// worked.
fn copy_history_db(old_dir: &Path, new_dir: &Path) -> bool {
    let src = old_dir.join("history.db");
    if !src.is_file() {
        return true;
    }
    let dst = new_dir.join("history.db");
    if dst.exists() {
        log::info!("migration: history.db already present in the new dir, keeping it");
        return true;
    }

    match legacy_schema_version(&src) {
        Ok(version) if version > crate::managers::history::schema_version() => {
            log::warn!(
                "migration: legacy history.db schema v{} is newer than supported v{}; skipping history (it stays in the legacy dir)",
                version,
                crate::managers::history::schema_version()
            );
            return true;
        }
        Ok(_) => {}
        Err(e) => {
            log::warn!(
                "migration: legacy history.db is not a readable SQLite database ({}); skipping it",
                e
            );
            return true;
        }
    }

    match sqlite_backup(&src, &dst) {
        Ok(()) => {
            log::info!("migration: history.db snapshotted via sqlite backup");
            true
        }
        Err(e) => {
            log::warn!(
                "migration: sqlite backup of history.db failed ({}), falling back to file copy",
                e
            );
            let ok = copy_file_with_retry(&src, &dst);
            if ok {
                copy_file_best_effort(
                    &old_dir.join("history.db-wal"),
                    &new_dir.join("history.db-wal"),
                );
            } else {
                // Don't leave a partial database for HistoryManager to choke on
                let _ = fs::remove_file(&dst);
            }
            ok
        }
    }
}

fn legacy_schema_version(src: &Path) -> Result<i32, rusqlite::Error> {
    use rusqlite::{Connection, OpenFlags};
    let conn = Connection::open_with_flags(src, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
}

fn sqlite_backup(src: &Path, dst: &Path) -> Result<(), rusqlite::Error> {
    use rusqlite::{backup::Backup, Connection, OpenFlags};
    let src_conn = Connection::open_with_flags(src, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let mut dst_conn = Connection::open(dst)?;
    let backup = Backup::new(&src_conn, &mut dst_conn)?;
    // No pause between steps: this is a one-shot local copy with no writer to
    // be polite to, and rusqlite sleeps after every step, not just on Busy
    backup.run_to_completion(1024, std::time::Duration::ZERO, None)?;
    Ok(())
}

/// Copy a single file with a few retries (transient AV/indexer locks).
/// Returns `false` only when the source exists and the copy failed for good.
fn copy_file_with_retry(src: &Path, dst: &Path) -> bool {
    if !src.is_file() {
        return true;
    }
    for attempt in 1..=3u32 {
        match fs::copy(src, dst) {
            Ok(_) => return true,
            Err(e) if attempt < 3 => {
                log::warn!(
                    "migration: copy {} failed (attempt {}): {}",
                    src.display(),
                    attempt,
                    e
                );
                std::thread::sleep(std::time::Duration::from_millis(200));
            }
            Err(e) => {
                log::error!(
                    "migration: giving up on {} after {} attempts: {}",
                    src.display(),
                    attempt,
                    e
                );
            }
        }
    }
    false
}

/// Copy a single optional file; never overwrites an existing destination,
/// failure is logged and tolerated.
fn copy_file_best_effort(src: &Path, dst: &Path) {
    if !src.is_file() || dst.exists() {
        return;
    }
    if let Err(e) = fs::copy(src, dst) {
        log::warn!("migration: failed to copy {}: {}", src.display(), e);
    }
}

/// Recursively copy a directory tree. Existing destination files are never
/// overwritten; symlinks are skipped; individual file failures are logged and
/// tolerated. Returns `false` only when the source exists but could not be
/// copied at all.
fn copy_dir_recursive(src: &Path, dst: &Path, depth: u32) -> bool {
    if !src.is_dir() {
        return true;
    }
    if depth > 8 {
        log::warn!("migration: depth cap reached at {}", src.display());
        return true;
    }
    if let Err(e) = fs::create_dir_all(dst) {
        log::error!("migration: cannot create {}: {}", dst.display(), e);
        return false;
    }
    let entries = match fs::read_dir(src) {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("migration: cannot read {}: {}", src.display(), e);
            return false;
        }
    };
    for entry in entries.flatten() {
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        let meta = match fs::symlink_metadata(&src_path) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("migration: cannot stat {}: {}", src_path.display(), e);
                continue;
            }
        };
        if meta.is_symlink() {
            log::warn!("migration: skipping symlink {}", src_path.display());
        } else if meta.is_dir() {
            copy_dir_recursive(&src_path, &dst_path, depth + 1);
        } else if dst_path.exists() {
            // Re-run safety: never clobber data the new app has written
        } else if let Err(e) = fs::copy(&src_path, &dst_path) {
            log::warn!("migration: failed to copy {}: {}", src_path.display(), e);
        }
    }
    true
}

/// Move directory entries one by one (same-volume rename, no copying of
/// gigabytes). One locked file then only costs one warning instead of failing
/// the whole move. Entries whose names end in one of `skip_suffixes`, and
/// entries already present in the destination, are left behind. Returns
/// `false` only when the destination could not be created or the source could
/// not be enumerated.
fn move_dir_entries(old: &Path, new: &Path, skip_suffixes: &[&str]) -> bool {
    if let Err(e) = fs::create_dir_all(new) {
        log::error!("migration: cannot create {}: {}", new.display(), e);
        return false;
    }
    let entries = match fs::read_dir(old) {
        Ok(entries) => entries,
        Err(e) => {
            log::error!("migration: cannot read {}: {}", old.display(), e);
            return false;
        }
    };
    let mut moved = 0u32;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_string();
        if skip_suffixes.iter().any(|s| name_str.ends_with(s)) {
            continue;
        }
        let dst = new.join(&name);
        if dst.exists() {
            log::warn!(
                "migration: {} already exists, leaving legacy copy in place",
                dst.display()
            );
            continue;
        }
        match fs::rename(entry.path(), &dst) {
            Ok(()) => moved += 1,
            Err(e) => log::warn!(
                "migration: could not move {} ({}); it stays in the legacy dir",
                name_str,
                e
            ),
        }
    }
    log::info!("migration: moved {} entries from {}", moved, old.display());
    true
}

/// Detect an installed copy of the original upstream Handy, which shared the
/// legacy data dir and must keep its models and recordings.
fn original_handy_installed() -> bool {
    #[cfg(target_os = "macos")]
    {
        if Path::new("/Applications/Handy.app").exists() {
            return true;
        }
        if let Some(home) = std::env::var_os("HOME") {
            if std::path::PathBuf::from(home)
                .join("Applications/Handy.app")
                .exists()
            {
                return true;
            }
        }
    }
    #[cfg(target_os = "windows")]
    {
        for (var, sub) in [
            ("LOCALAPPDATA", "Handy\\handy.exe"),
            ("ProgramFiles", "Handy\\handy.exe"),
        ] {
            if let Some(base) = std::env::var_os(var) {
                if std::path::PathBuf::from(base).join(sub).exists() {
                    return true;
                }
            }
        }
    }
    // Linux (AppImage installs are undetectable): assume absent — a wrong
    // guess only costs the other app a model re-download
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("handy_migration_{}", name));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write_file(path: &Path, content: &str) {
        let mut f = std::fs::File::create(path).unwrap();
        write!(f, "{}", content).unwrap();
    }

    fn create_sqlite_db(path: &Path, value: &str) {
        let conn = rusqlite::Connection::open(path).unwrap();
        conn.execute_batch(&format!(
            "CREATE TABLE t (v TEXT); INSERT INTO t (v) VALUES ('{}');",
            value
        ))
        .unwrap();
    }

    fn read_sqlite_value(path: &Path) -> String {
        let conn = rusqlite::Connection::open(path).unwrap();
        conn.query_row("SELECT v FROM t", [], |row| row.get(0))
            .unwrap()
    }

    /// Build a realistic legacy data dir: settings, real SQLite history,
    /// recordings, custom sound, models with a real entry and interrupted
    /// leftovers.
    fn seed_legacy_dir(old: &Path) {
        std::fs::create_dir_all(old.join("recordings")).unwrap();
        std::fs::create_dir_all(old.join("models")).unwrap();
        write_file(&old.join("settings_store.json"), "{\"settings\":{}}");
        create_sqlite_db(&old.join("history.db"), "legacy");
        write_file(&old.join("recordings").join("handy-123.wav"), "wav");
        write_file(&old.join("custom_start.wav"), "start");
        write_file(&old.join("models").join("ggml-small.bin"), "model");
        write_file(&old.join("models").join("foo.bin.partial"), "partial");
    }

    #[test]
    fn test_copy_dir_recursive_copies_nested_tree() {
        let root = temp_dir("copy_nested");
        let src = root.join("src");
        std::fs::create_dir_all(src.join("sub")).unwrap();
        write_file(&src.join("a.wav"), "aaa");
        write_file(&src.join("sub").join("b.wav"), "bbb");

        let dst = root.join("dst");
        assert!(copy_dir_recursive(&src, &dst, 0));
        assert_eq!(std::fs::read_to_string(dst.join("a.wav")).unwrap(), "aaa");
        assert_eq!(
            std::fs::read_to_string(dst.join("sub").join("b.wav")).unwrap(),
            "bbb"
        );
        // Source untouched
        assert!(src.join("a.wav").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_copy_dir_recursive_never_overwrites_existing_files() {
        let root = temp_dir("copy_no_clobber");
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        write_file(&src.join("a.wav"), "legacy");

        let dst = root.join("dst");
        std::fs::create_dir_all(&dst).unwrap();
        write_file(&dst.join("a.wav"), "new-data");

        assert!(copy_dir_recursive(&src, &dst, 0));
        assert_eq!(
            std::fs::read_to_string(dst.join("a.wav")).unwrap(),
            "new-data"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_copy_dir_recursive_missing_source_is_ok() {
        let root = temp_dir("copy_missing");
        assert!(copy_dir_recursive(
            &root.join("nonexistent"),
            &root.join("dst"),
            0
        ));
        assert!(!root.join("dst").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_copy_dir_recursive_skips_symlinks() {
        let root = temp_dir("copy_symlink");
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        write_file(&src.join("real.wav"), "real");
        std::os::unix::fs::symlink(src.join("real.wav"), src.join("link.wav")).unwrap();

        let dst = root.join("dst");
        assert!(copy_dir_recursive(&src, &dst, 0));
        assert!(dst.join("real.wav").exists());
        assert!(!dst.join("link.wav").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_move_dir_entries_skips_suffixes() {
        let root = temp_dir("move_partial");
        let old = root.join("models");
        std::fs::create_dir_all(&old).unwrap();
        write_file(&old.join("ggml-small.bin"), "model");
        write_file(&old.join("foo.bin.partial"), "partial");
        std::fs::create_dir_all(old.join("bar.extracting")).unwrap();

        let new = root.join("new_models");
        assert!(move_dir_entries(&old, &new, &[".partial", ".extracting"]));

        assert!(new.join("ggml-small.bin").exists());
        assert!(!old.join("ggml-small.bin").exists());
        // Leftovers stay behind and are not moved
        assert!(old.join("foo.bin.partial").exists());
        assert!(old.join("bar.extracting").exists());
        assert!(!new.join("foo.bin.partial").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_move_dir_entries_merges_into_existing_destination() {
        let root = temp_dir("move_merge");
        let old = root.join("models");
        std::fs::create_dir_all(&old).unwrap();
        write_file(&old.join("existing.bin"), "old-version");
        write_file(&old.join("fresh.bin"), "fresh");
        std::fs::create_dir_all(old.join("parakeet-dir")).unwrap();
        write_file(&old.join("parakeet-dir").join("weights"), "w");

        let new = root.join("new_models");
        std::fs::create_dir_all(&new).unwrap();
        write_file(&new.join("existing.bin"), "new-version");

        assert!(move_dir_entries(&old, &new, &[]));

        // Existing destination entry wins, legacy copy stays put
        assert_eq!(
            std::fs::read_to_string(new.join("existing.bin")).unwrap(),
            "new-version"
        );
        assert!(old.join("existing.bin").exists());
        // Fresh entries (file and directory) are moved
        assert!(new.join("fresh.bin").exists());
        assert!(!old.join("fresh.bin").exists());
        assert!(new.join("parakeet-dir").join("weights").exists());
        assert!(!old.join("parakeet-dir").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_full_flow_moves_models_and_keeps_backup() {
        let root = temp_dir("full_flow");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        seed_legacy_dir(&old);

        migrate_between(&old, &new, false);

        // Settings and history are copied, marker landed
        assert_eq!(
            std::fs::read_to_string(new.join("settings_store.json")).unwrap(),
            "{\"settings\":{}}"
        );
        assert_eq!(read_sqlite_value(&new.join("history.db")), "legacy");
        assert!(new.join("custom_start.wav").exists());
        // Recordings and models moved, leftovers stay behind
        assert!(new.join("recordings").join("handy-123.wav").exists());
        assert!(!old.join("recordings").join("handy-123.wav").exists());
        assert!(new.join("models").join("ggml-small.bin").exists());
        assert!(!old.join("models").join("ggml-small.bin").exists());
        assert!(old.join("models").join("foo.bin.partial").exists());
        // Legacy settings and history stay behind as a backup
        assert!(old.join("settings_store.json").exists());
        assert_eq!(read_sqlite_value(&old.join("history.db")), "legacy");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_leaves_models_and_recordings_when_handy_installed() {
        let root = temp_dir("handy_installed");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        seed_legacy_dir(&old);

        migrate_between(&old, &new, true);

        // Models stay for Handy; recordings are copied, not moved
        assert!(old.join("models").join("ggml-small.bin").exists());
        assert!(!new.join("models").exists());
        assert!(old.join("recordings").join("handy-123.wav").exists());
        assert!(new.join("recordings").join("handy-123.wav").exists());
        // Small items still migrated
        assert!(new.join("settings_store.json").exists());
        assert!(new.join("history.db").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_rerun_does_not_clobber_new_data() {
        let root = temp_dir("rerun");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        seed_legacy_dir(&old);

        migrate_between(&old, &new, true);

        // The new app worked for a while: fresh history and a new recording
        write_file(&new.join("history.db"), "NEW-DB");
        write_file(&new.join("recordings").join("handy-123.wav"), "NEW-WAV");

        // Marker gone (e.g. user reset settings) -> migration runs again
        std::fs::remove_file(new.join("settings_store.json")).unwrap();
        migrate_between(&old, &new, true);

        assert_eq!(
            std::fs::read_to_string(new.join("history.db")).unwrap(),
            "NEW-DB"
        );
        assert_eq!(
            std::fs::read_to_string(new.join("recordings").join("handy-123.wav")).unwrap(),
            "NEW-WAV"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_withholds_marker_when_recordings_fail() {
        let root = temp_dir("gate");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        seed_legacy_dir(&old);
        // Occupy the recordings destination with a file so the move fails
        std::fs::create_dir_all(&new).unwrap();
        write_file(&new.join("recordings"), "in the way");

        migrate_between(&old, &new, false);

        assert!(!new.join("settings_store.json").exists());
        assert!(!new.join("settings_store.json.migrating").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_models_only_legacy_dir() {
        // User wiped settings once: models present, no settings/history
        let root = temp_dir("models_only");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        std::fs::create_dir_all(old.join("models")).unwrap();
        write_file(&old.join("models").join("ggml-small.bin"), "model");

        migrate_between(&old, &new, false);

        assert!(new.join("models").join("ggml-small.bin").exists());
        assert!(!new.join("settings_store.json").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_tolerates_preexisting_new_dir() {
        // On Linux the log plugin creates <new>/logs before migration runs
        let root = temp_dir("preexisting_new");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        seed_legacy_dir(&old);
        std::fs::create_dir_all(new.join("logs")).unwrap();
        write_file(&new.join("logs").join("handy.log"), "log");

        migrate_between(&old, &new, false);

        assert!(new.join("settings_store.json").exists());
        assert!(new.join("logs").join("handy.log").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_copy_history_db_snapshots_real_sqlite_db() {
        let root = temp_dir("sqlite_backup");
        let old = root.join("old");
        let new = root.join("new");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        create_sqlite_db(&old.join("history.db"), "hello");

        assert!(copy_history_db(&old, &new));
        assert_eq!(read_sqlite_value(&new.join("history.db")), "hello");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_copy_history_db_snapshots_wal_mode_db() {
        // Old DBs created via tauri-plugin-sql stay in WAL mode
        let root = temp_dir("sqlite_wal");
        let old = root.join("old");
        let new = root.join("new");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        {
            let conn = rusqlite::Connection::open(old.join("history.db")).unwrap();
            conn.pragma_update(None, "journal_mode", "WAL").unwrap();
            conn.execute_batch("CREATE TABLE t (v TEXT); INSERT INTO t (v) VALUES ('wal');")
                .unwrap();
        }

        assert!(copy_history_db(&old, &new));
        assert_eq!(read_sqlite_value(&new.join("history.db")), "wal");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_copy_history_db_skips_non_sqlite_file() {
        let root = temp_dir("sqlite_garbage");
        let old = root.join("old");
        let new = root.join("new");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        write_file(&old.join("history.db"), "not a database");

        // Skipped (returns true so the rest of the migration proceeds), and
        // the unreadable file is not brought over to brick HistoryManager
        assert!(copy_history_db(&old, &new));
        assert!(!new.join("history.db").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_copy_history_db_skips_newer_schema() {
        let root = temp_dir("sqlite_newer");
        let old = root.join("old");
        let new = root.join("new");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        {
            let conn = rusqlite::Connection::open(old.join("history.db")).unwrap();
            conn.execute_batch("CREATE TABLE t (v TEXT);").unwrap();
            conn.pragma_update(None, "user_version", 999).unwrap();
        }

        assert!(copy_history_db(&old, &new));
        assert!(!new.join("history.db").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_copy_history_db_keeps_existing_destination() {
        let root = temp_dir("sqlite_existing");
        let old = root.join("old");
        let new = root.join("new");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        create_sqlite_db(&old.join("history.db"), "legacy");
        write_file(&new.join("history.db"), "NEW-DB");

        assert!(copy_history_db(&old, &new));
        assert_eq!(
            std::fs::read_to_string(new.join("history.db")).unwrap(),
            "NEW-DB"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_copy_file_helpers_tolerate_missing_source() {
        let root = temp_dir("copy_file_missing");
        assert!(copy_file_with_retry(
            &root.join("nope.db"),
            &root.join("dst.db")
        ));
        copy_file_best_effort(&root.join("nope.wav"), &root.join("dst.wav"));
        assert!(!root.join("dst.db").exists());
        assert!(!root.join("dst.wav").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}

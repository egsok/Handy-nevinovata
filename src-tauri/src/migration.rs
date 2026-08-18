//! One-time migration of user data from the legacy `com.pais.handy` app data
//! directory into the current identifier's directory.
//!
//! The legacy identifier was shared with the original upstream Handy app, so
//! the legacy directory may still be in use by it: settings and history are
//! copied, models and recordings are hard-linked (same-volume by construction,
//! so this is instant and costs no disk; both apps can keep using the files),
//! and the legacy directory is never deleted. Nothing already present in the
//! new directory is ever overwritten, so a re-run cannot clobber data the new
//! app has written since.
//!
//! Completion is recorded in a dedicated marker file, written only when the
//! important items (history, recordings) made it across. A transient failure
//! (e.g. an antivirus holding history.db on first launch) therefore gets a
//! real retry on the next launch — the app will have created its own settings
//! by then, but history and recordings are still picked up.

use std::fs;
use std::path::Path;

const LEGACY_IDENTIFIER: &str = "com.pais.handy";
const SETTINGS_FILE: &str = "settings_store.json";
const MARKER_FILE: &str = ".migrated-from-com.pais.handy";

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

    if new_dir.join(MARKER_FILE).exists() {
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

    migrate_between(&old_dir, &new_dir);
}

/// Path-based core of the migration, separated from AppHandle resolution so
/// the whole flow is unit-testable against temp directories.
fn migrate_between(old_dir: &Path, new_dir: &Path) {
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

    // Recordings and models are hard-linked, not moved or copied: instant,
    // zero extra disk, and an upstream Handy still using the legacy dir keeps
    // every file it references. Both apps only ever write these files once
    // under unique names, so sharing the content is safe.
    let old_recordings = old_dir.join("recordings");
    let ok_recordings = if old_recordings.is_dir() {
        link_dir_recursive(&old_recordings, &new_dir.join("recordings"), 0, &[])
    } else {
        true
    };

    for sound in ["custom_start.wav", "custom_stop.wav"] {
        copy_file_best_effort(&old_dir.join(sound), &new_dir.join(sound));
    }

    let old_models = old_dir.join("models");
    if old_models.is_dir() {
        link_dir_recursive(
            &old_models,
            &new_dir.join("models"),
            0,
            &[".partial", ".extracting"],
        );
    }

    if ok_history && ok_recordings {
        copy_settings(old_dir, new_dir);
        // Only now is the migration recorded as done. A missing marker means
        // the whole (idempotent) flow runs again on the next launch.
        if let Err(e) = fs::write(new_dir.join(MARKER_FILE), b"") {
            log::warn!("migration: cannot write completion marker: {}", e);
        }
        log::info!(
            "migration: complete; legacy data kept at {}",
            old_dir.display()
        );
    } else {
        log::error!(
            "migration: important items failed; will retry on next launch. Legacy data remains at {}",
            old_dir.display()
        );
    }
}

/// Copy the legacy settings file, never overwriting one the app has already
/// written (on a retry run the app has long since created its own defaults —
/// those may have been edited and must win).
fn copy_settings(old_dir: &Path, new_dir: &Path) {
    let old_settings = old_dir.join(SETTINGS_FILE);
    if !old_settings.is_file() {
        log::info!("migration: legacy dir has no {}", SETTINGS_FILE);
        return;
    }
    let dst = new_dir.join(SETTINGS_FILE);
    if dst.exists() {
        log::info!(
            "migration: {} already present in the new dir, keeping it",
            SETTINGS_FILE
        );
        return;
    }
    // Copy via temp file + rename so an interrupted copy cannot leave a
    // partial settings file for the store plugin to choke on
    let tmp = new_dir.join("settings_store.json.migrating");
    let result = fs::copy(&old_settings, &tmp).and_then(|_| fs::rename(&tmp, &dst));
    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        log::error!(
            "migration: failed to copy settings: {}; defaults will apply, legacy copy remains at {}",
            e,
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

    match check_legacy_schema(&src) {
        SchemaCheck::Newer(version) => {
            log::warn!(
                "migration: legacy history.db schema v{} is newer than supported v{}; skipping history (it stays in the legacy dir)",
                version,
                crate::managers::history::schema_version()
            );
            return true;
        }
        SchemaCheck::Supported => {}
        SchemaCheck::NotADatabase(e) => {
            log::warn!(
                "migration: legacy history.db is not a usable SQLite database ({}); skipping it",
                e
            );
            return true;
        }
        SchemaCheck::Unreadable(e) => {
            // Transient I/O failure (AV lock, permissions): a real database we
            // simply could not read right now. Report failure so the completion
            // marker is withheld and the next launch retries.
            log::error!(
                "migration: cannot open legacy history.db ({}); history not migrated, will retry",
                e
            );
            return false;
        }
    }

    let backup_failure = match sqlite_backup(&src, &dst) {
        Ok(()) => {
            log::info!("migration: history.db snapshotted via sqlite backup");
            return true;
        }
        Err(failure) => failure,
    };
    log::warn!(
        "migration: sqlite backup of history.db failed ({}), falling back to file copy",
        backup_failure
    );
    // Copy the WAL sidecar first: capturing it before the main file can only
    // make the replayed copy older than the main file, never newer
    copy_file_best_effort(
        &old_dir.join("history.db-wal"),
        &new_dir.join("history.db-wal"),
    );
    if !copy_file_with_retry(&src, &dst) {
        // Don't leave a partial database for HistoryManager to choke on
        let _ = fs::remove_file(&dst);
        let _ = fs::remove_file(new_dir.join("history.db-wal"));
        return false;
    }
    // A plain copy taken while the legacy app was writing can be torn; never
    // hand HistoryManager a corrupt database
    if !copied_db_is_sound(&dst) {
        let _ = fs::remove_file(&dst);
        let _ = fs::remove_file(new_dir.join("history.db-wal"));
        return match backup_failure {
            // The backup gave up on a lock, so the tear is most likely the
            // concurrent writer — a retry on the next launch can succeed
            BackupFailure::LockTimeout => {
                log::error!(
                    "migration: file copy of history.db is not consistent (legacy app still writing?); will retry"
                );
                false
            }
            // The backup saw a real database error: the source itself is bad,
            // retrying will not help. Drop the history, migrate everything else.
            BackupFailure::Failed(_) => {
                log::error!(
                    "migration: legacy history.db appears damaged; skipping history (it stays in the legacy dir)"
                );
                true
            }
        };
    }
    true
}

enum SchemaCheck {
    Supported,
    Newer(i32),
    /// Permanently unusable (not SQLite, or corrupt): skip it and migrate on
    NotADatabase(String),
    /// Could not be read right now (lock, permissions): treat as failure
    Unreadable(String),
}

fn check_legacy_schema(src: &Path) -> SchemaCheck {
    let mut last_err = String::new();
    for attempt in 1..=3u32 {
        match legacy_schema_version(src) {
            Ok(v) if v > crate::managers::history::schema_version() => {
                return SchemaCheck::Newer(v)
            }
            Ok(_) => return SchemaCheck::Supported,
            Err(e) if is_permanently_unusable(&e) => {
                return SchemaCheck::NotADatabase(e.to_string())
            }
            Err(e) => {
                last_err = e.to_string();
                if attempt < 3 {
                    log::warn!(
                        "migration: cannot open legacy history.db (attempt {}): {}",
                        attempt,
                        last_err
                    );
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
        }
    }
    SchemaCheck::Unreadable(last_err)
}

fn is_permanently_unusable(e: &rusqlite::Error) -> bool {
    matches!(
        e,
        rusqlite::Error::SqliteFailure(err, _) if matches!(
            err.code,
            rusqlite::ErrorCode::NotADatabase | rusqlite::ErrorCode::DatabaseCorrupt
        )
    )
}

fn legacy_schema_version(src: &Path) -> Result<i32, rusqlite::Error> {
    use rusqlite::{Connection, OpenFlags};
    let conn = Connection::open_with_flags(src, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    conn.pragma_query_value(None, "user_version", |row| row.get(0))
}

enum BackupFailure {
    /// Could not finish within the deadline (source locked or restarted by a
    /// concurrent writer) — transient
    LockTimeout,
    /// A real SQLite error from the source database — permanent
    Failed(String),
}

impl std::fmt::Display for BackupFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupFailure::LockTimeout => f.write_str("timed out waiting for the database lock"),
            BackupFailure::Failed(e) => f.write_str(e),
        }
    }
}

fn sqlite_backup(src: &Path, dst: &Path) -> Result<(), BackupFailure> {
    use rusqlite::backup::{Backup, StepResult};
    use rusqlite::{Connection, OpenFlags};
    let fail = |e: rusqlite::Error| BackupFailure::Failed(e.to_string());
    let src_conn =
        Connection::open_with_flags(src, OpenFlags::SQLITE_OPEN_READ_ONLY).map_err(fail)?;
    let mut dst_conn = Connection::open(dst).map_err(fail)?;
    let backup = Backup::new(&src_conn, &mut dst_conn).map_err(fail)?;
    // Stepped manually instead of run_to_completion: with the legacy app still
    // writing, run_to_completion retries Busy/Locked forever (and with a zero
    // pause would spin at full CPU) right on the startup path. The deadline is
    // checked on every iteration — an external writer restarts the backup from
    // page 1, so even a steady stream of More results must be bounded.
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        if std::time::Instant::now() >= deadline {
            return Err(BackupFailure::LockTimeout);
        }
        match backup.step(1024).map_err(fail)? {
            StepResult::Done => return Ok(()),
            StepResult::More => {}
            // The enum is non_exhaustive; treat unknown results like Busy
            StepResult::Busy | StepResult::Locked | _ => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }
}

/// `PRAGMA quick_check` on a freshly copied database; anything but a clean
/// "ok" means the copy must not be used.
fn copied_db_is_sound(db: &Path) -> bool {
    let result: Result<String, rusqlite::Error> = (|| {
        let conn = rusqlite::Connection::open(db)?;
        conn.query_row("PRAGMA quick_check", [], |row| row.get(0))
    })();
    match result {
        Ok(ref verdict) if verdict == "ok" => true,
        Ok(verdict) => {
            log::warn!("migration: quick_check on copied history.db: {}", verdict);
            false
        }
        Err(e) => {
            log::warn!("migration: quick_check on copied history.db failed: {}", e);
            false
        }
    }
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

/// Recursively hard-link a directory tree into `dst` (falling back to a plain
/// copy on filesystems without hard links). The legacy tree is left fully in
/// place, so an upstream Handy still using it loses nothing, while the link
/// itself is instant and costs no disk. Entries whose names end in one of
/// `skip_suffixes`, existing destination files (re-run safety) and symlinks
/// are left alone. Returns `false` when any file could not be brought across.
fn link_dir_recursive(src: &Path, dst: &Path, depth: u32, skip_suffixes: &[&str]) -> bool {
    if !src.is_dir() {
        return true;
    }
    if depth > 8 {
        // Anything this deep is not a layout the app ever produced; report
        // failure rather than pretending the truncated tree is complete
        log::warn!("migration: depth cap reached at {}", src.display());
        return false;
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
    let mut ok = true;
    for entry in entries.flatten() {
        let src_path = entry.path();
        let name = entry.file_name();
        if skip_suffixes
            .iter()
            .any(|s| name.to_string_lossy().ends_with(s))
        {
            continue;
        }
        let dst_path = dst.join(&name);
        let meta = match fs::symlink_metadata(&src_path) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("migration: cannot stat {}: {}", src_path.display(), e);
                ok = false;
                continue;
            }
        };
        if meta.is_symlink() {
            log::warn!("migration: skipping symlink {}", src_path.display());
        } else if meta.is_dir() {
            ok &= link_dir_recursive(&src_path, &dst_path, depth + 1, skip_suffixes);
        } else if dst_path.exists() {
            // Re-run safety: never clobber data the new app has written
        } else if let Err(link_err) = fs::hard_link(&src_path, &dst_path) {
            // e.g. exFAT has no hard links; fall back to copying this file
            if let Err(copy_err) = fs::copy(&src_path, &dst_path) {
                log::warn!(
                    "migration: failed to link ({}) or copy ({}) {}",
                    link_err,
                    copy_err,
                    src_path.display()
                );
                ok = false;
            }
        }
    }
    ok
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
    fn test_link_dir_recursive_links_nested_tree_and_keeps_source() {
        let root = temp_dir("link_nested");
        let src = root.join("src");
        std::fs::create_dir_all(src.join("sub")).unwrap();
        write_file(&src.join("a.wav"), "aaa");
        write_file(&src.join("sub").join("b.wav"), "bbb");

        let dst = root.join("dst");
        assert!(link_dir_recursive(&src, &dst, 0, &[]));
        assert_eq!(std::fs::read_to_string(dst.join("a.wav")).unwrap(), "aaa");
        assert_eq!(
            std::fs::read_to_string(dst.join("sub").join("b.wav")).unwrap(),
            "bbb"
        );
        // Source stays fully in place (legacy dir doubles as backup)
        assert_eq!(std::fs::read_to_string(src.join("a.wav")).unwrap(), "aaa");
        assert!(src.join("sub").join("b.wav").exists());
        // Pin the hard-link semantics, not just the copied content: reverting
        // to fs::copy must fail this
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let src_meta = std::fs::metadata(src.join("a.wav")).unwrap();
            let dst_meta = std::fs::metadata(dst.join("a.wav")).unwrap();
            assert_eq!(src_meta.ino(), dst_meta.ino());
            assert_eq!(src_meta.nlink(), 2);
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_link_dir_recursive_never_overwrites_existing_files() {
        let root = temp_dir("link_no_clobber");
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        write_file(&src.join("a.wav"), "legacy");

        let dst = root.join("dst");
        std::fs::create_dir_all(&dst).unwrap();
        write_file(&dst.join("a.wav"), "new-data");

        assert!(link_dir_recursive(&src, &dst, 0, &[]));
        assert_eq!(
            std::fs::read_to_string(dst.join("a.wav")).unwrap(),
            "new-data"
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_link_dir_recursive_missing_source_is_ok() {
        let root = temp_dir("link_missing");
        assert!(link_dir_recursive(
            &root.join("nonexistent"),
            &root.join("dst"),
            0,
            &[]
        ));
        assert!(!root.join("dst").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn test_link_dir_recursive_skips_symlinks() {
        let root = temp_dir("link_symlink");
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        write_file(&src.join("real.wav"), "real");
        std::os::unix::fs::symlink(src.join("real.wav"), src.join("link.wav")).unwrap();

        let dst = root.join("dst");
        assert!(link_dir_recursive(&src, &dst, 0, &[]));
        assert!(dst.join("real.wav").exists());
        assert!(!dst.join("link.wav").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_link_dir_recursive_skips_suffixes() {
        let root = temp_dir("link_partial");
        let old = root.join("models");
        std::fs::create_dir_all(&old).unwrap();
        write_file(&old.join("ggml-small.bin"), "model");
        write_file(&old.join("foo.bin.partial"), "partial");
        std::fs::create_dir_all(old.join("bar.extracting")).unwrap();

        let new = root.join("new_models");
        assert!(link_dir_recursive(
            &old,
            &new,
            0,
            &[".partial", ".extracting"]
        ));

        assert!(new.join("ggml-small.bin").exists());
        // Interrupted leftovers are not brought over
        assert!(!new.join("foo.bin.partial").exists());
        assert!(!new.join("bar.extracting").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_link_dir_recursive_merges_into_existing_destination() {
        let root = temp_dir("link_merge");
        let old = root.join("models");
        std::fs::create_dir_all(&old).unwrap();
        write_file(&old.join("existing.bin"), "old-version");
        write_file(&old.join("fresh.bin"), "fresh");
        std::fs::create_dir_all(old.join("parakeet-dir")).unwrap();
        write_file(&old.join("parakeet-dir").join("weights"), "w");

        let new = root.join("new_models");
        std::fs::create_dir_all(&new).unwrap();
        write_file(&new.join("existing.bin"), "new-version");

        assert!(link_dir_recursive(&old, &new, 0, &[]));

        // Existing destination entry wins
        assert_eq!(
            std::fs::read_to_string(new.join("existing.bin")).unwrap(),
            "new-version"
        );
        assert!(old.join("existing.bin").exists());
        // Fresh entries (file and nested directory) are linked across
        assert!(new.join("fresh.bin").exists());
        assert!(new.join("parakeet-dir").join("weights").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_full_flow_links_data_and_keeps_backup() {
        let root = temp_dir("full_flow");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        seed_legacy_dir(&old);

        migrate_between(&old, &new);

        // Settings and history are copied, marker landed
        assert_eq!(
            std::fs::read_to_string(new.join("settings_store.json")).unwrap(),
            "{\"settings\":{}}"
        );
        assert_eq!(read_sqlite_value(&new.join("history.db")), "legacy");
        assert!(new.join("custom_start.wav").exists());
        // Recordings and models are linked across; interrupted leftovers are not
        assert!(new.join("recordings").join("handy-123.wav").exists());
        assert!(new.join("models").join("ggml-small.bin").exists());
        assert!(!new.join("models").join("foo.bin.partial").exists());
        // The whole legacy dir stays fully in place as a backup (and for a
        // still-installed upstream Handy)
        assert!(old.join("settings_store.json").exists());
        assert!(old.join("recordings").join("handy-123.wav").exists());
        assert!(old.join("models").join("ggml-small.bin").exists());
        assert!(old.join("models").join("foo.bin.partial").exists());
        assert_eq!(read_sqlite_value(&old.join("history.db")), "legacy");
        // Completion marker landed
        assert!(new.join(MARKER_FILE).exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_rerun_does_not_clobber_new_data() {
        let root = temp_dir("rerun");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        seed_legacy_dir(&old);

        migrate_between(&old, &new);

        // The new app worked for a while: fresh history, edited settings and a
        // replaced recording (removed first — the migrated file is a hard
        // link, and writing through it would also change the legacy copy)
        std::fs::remove_file(new.join("history.db")).unwrap();
        write_file(&new.join("history.db"), "NEW-DB");
        std::fs::remove_file(new.join("recordings").join("handy-123.wav")).unwrap();
        write_file(&new.join("recordings").join("handy-123.wav"), "NEW-WAV");
        std::fs::remove_file(new.join("settings_store.json")).unwrap();
        write_file(&new.join("settings_store.json"), "{\"edited\":true}");

        // Marker gone -> migration runs again
        std::fs::remove_file(new.join(MARKER_FILE)).unwrap();
        migrate_between(&old, &new);

        assert_eq!(
            std::fs::read_to_string(new.join("history.db")).unwrap(),
            "NEW-DB"
        );
        assert_eq!(
            std::fs::read_to_string(new.join("recordings").join("handy-123.wav")).unwrap(),
            "NEW-WAV"
        );
        // The app's own settings win over the legacy ones on a re-run
        assert_eq!(
            std::fs::read_to_string(new.join("settings_store.json")).unwrap(),
            "{\"edited\":true}"
        );
        assert!(new.join(MARKER_FILE).exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_withholds_marker_when_recordings_fail() {
        let root = temp_dir("gate");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        seed_legacy_dir(&old);
        // Occupy the recordings destination with a file so the link fails
        std::fs::create_dir_all(&new).unwrap();
        write_file(&new.join("recordings"), "in the way");

        migrate_between(&old, &new);

        assert!(!new.join(MARKER_FILE).exists());
        assert!(!new.join("settings_store.json").exists());
        assert!(!new.join("settings_store.json.migrating").exists());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_migrate_between_retries_after_transient_failure() {
        // First run fails on recordings; the blocker goes away and the next
        // launch picks the history and recordings up
        let root = temp_dir("retry");
        let old = root.join("com.pais.handy");
        let new = root.join("ru.egorsokolov.klava-nevinovata");
        seed_legacy_dir(&old);
        std::fs::create_dir_all(&new).unwrap();
        write_file(&new.join("recordings"), "in the way");

        migrate_between(&old, &new);
        assert!(!new.join(MARKER_FILE).exists());
        // The app meanwhile created its own settings (defaults)
        write_file(&new.join("settings_store.json"), "{\"defaults\":true}");

        std::fs::remove_file(new.join("recordings")).unwrap();
        migrate_between(&old, &new);

        assert!(new.join(MARKER_FILE).exists());
        assert_eq!(read_sqlite_value(&new.join("history.db")), "legacy");
        assert!(new.join("recordings").join("handy-123.wav").exists());
        // Settings created since the failed run are kept, not overwritten
        assert_eq!(
            std::fs::read_to_string(new.join("settings_store.json")).unwrap(),
            "{\"defaults\":true}"
        );
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

        migrate_between(&old, &new);

        assert!(new.join("models").join("ggml-small.bin").exists());
        assert!(!new.join("settings_store.json").exists());
        // Marker still lands so later launches skip the whole flow
        assert!(new.join(MARKER_FILE).exists());
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

        migrate_between(&old, &new);

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

    #[cfg(windows)]
    #[test]
    fn test_copy_history_db_reports_failure_on_locked_db() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = temp_dir("sqlite_locked");
        let old = root.join("old");
        let new = root.join("new");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        create_sqlite_db(&old.join("history.db"), "locked");

        // Exclusive handle with no sharing, like an AV/backup agent would hold
        let _guard = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(old.join("history.db"))
            .unwrap();

        // A real database we cannot read is a failure (marker must be
        // withheld), not a silent "skip the history"
        assert!(!copy_history_db(&old, &new));
        assert!(!new.join("history.db").exists());
        drop(_guard);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn test_copied_db_is_sound_rejects_truncated_db() {
        let root = temp_dir("sqlite_torn");
        let db = root.join("history.db");
        {
            let conn = rusqlite::Connection::open(&db).unwrap();
            // Force several pages so truncation actually tears the file
            conn.execute_batch("CREATE TABLE t (v BLOB);").unwrap();
            let blob = vec![0u8; 16384];
            conn.execute("INSERT INTO t (v) VALUES (?1)", [&blob])
                .unwrap();
        }
        assert!(copied_db_is_sound(&db));

        // Tear off everything past the first page, like a copy taken while
        // the legacy app was still writing
        let f = std::fs::OpenOptions::new().write(true).open(&db).unwrap();
        f.set_len(1024).unwrap();
        drop(f);
        assert!(!copied_db_is_sound(&db));
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

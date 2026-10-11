use super::*;

fn entry(status: Status, fresh_until: Timestamp) -> Entry {
    Entry {
        id: "a".to_owned(),
        note: "n".to_owned(),
        status,
        detail: "d".to_owned(),
        checked_at: Timestamp::UNIX_EPOCH,
        fresh_until,
        spec: "state = \"github:o/r#1\"\n".to_owned(),
    }
}

#[test]
fn a_failure_is_never_fresh() {
    let later = Timestamp::UNIX_EPOCH
        .checked_add(SignedDuration::from_hours(1))
        .expect("in range");
    assert!(entry(Status::Waiting, later).is_fresh(Timestamp::UNIX_EPOCH));
    assert!(!entry(Status::Waiting, later).is_fresh(later));
    assert!(!entry(Status::Failed, later).is_fresh(Timestamp::UNIX_EPOCH));
    let ttl = SignedDuration::from_hours(1);
    assert_eq!(
        Entry::freshness(Status::Waiting, Timestamp::UNIX_EPOCH, ttl),
        later
    );
    assert_eq!(
        Entry::freshness(Status::Failed, Timestamp::UNIX_EPOCH, ttl),
        Timestamp::UNIX_EPOCH
    );
    assert_eq!(
        Entry::freshness(Status::Waiting, Timestamp::MAX, ttl),
        Timestamp::MAX
    );
}

#[test]
fn the_default_path_mirrors_the_manifest_path() {
    assert_eq!(
        default_path(Path::new("/c"), Path::new("/home/me/.nix/canibump.toml")),
        PathBuf::from("/c/canibump/state/home/me/.nix/canibump.toml.json")
    );
}

#[test]
fn the_cache_home_prefers_xdg_then_home_then_localappdata() {
    let env = |vars: &'static [(&'static str, &'static str)]| {
        move |key: &str| {
            vars.iter()
                .find(|(name, _)| *name == key)
                .map(|(_, value)| OsString::from(value))
        }
    };
    assert_eq!(
        cache_home(env(&[("XDG_CACHE_HOME", "/x"), ("HOME", "/h")])),
        Some(PathBuf::from("/x"))
    );
    assert_eq!(
        cache_home(env(&[("XDG_CACHE_HOME", ""), ("HOME", "/h")])),
        Some(PathBuf::from("/h/.cache"))
    );
    assert_eq!(
        cache_home(env(&[("HOME", "/h"), ("LOCALAPPDATA", "/l")])),
        Some(PathBuf::from("/h/.cache"))
    );
    assert_eq!(
        cache_home(env(&[("HOME", ""), ("LOCALAPPDATA", "/l")])),
        Some(PathBuf::from("/l"))
    );
    assert_eq!(cache_home(env(&[])), None);
}

#[test]
fn round_trips_and_reports_a_missing_or_broken_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("deep/state.json");
    assert_eq!(read(&path).expect("missing is fine"), None);
    let state = StateFile {
        version: VERSION,
        manifest: PathBuf::from("/m/canibump.toml"),
        entries: vec![entry(Status::Bumpable, Timestamp::UNIX_EPOCH)],
    };
    write(&path, &state).expect("write");
    assert_eq!(read(&path).expect("read"), Some(state));
    std::fs::write(&path, "{").expect("break it");
    let error = read(&path).expect_err("broken").to_string();
    assert!(error.contains("not a canibump state file"), "{error}");
}

fn state() -> StateFile {
    StateFile {
        version: VERSION,
        manifest: PathBuf::from("/m/canibump.toml"),
        entries: vec![entry(Status::Waiting, Timestamp::UNIX_EPOCH)],
    }
}

#[test]
fn another_writers_temp_file_is_left_alone() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("state.json");
    let other = dir.path().join("state.json.tmp");
    std::fs::write(&other, "other writer").expect("another run is mid-write");
    write(&path, &state()).expect("write");
    assert_eq!(
        std::fs::read_to_string(&other).expect("still there"),
        "other writer"
    );
    assert_eq!(read(&path).expect("read"), Some(state()));
}

#[test]
fn a_failed_rename_leaves_no_temp_file_behind() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("state.json");
    std::fs::create_dir(&path).expect("a directory where the file belongs");
    write(&path, &state()).expect_err("a file cannot replace a directory");
    let left: Vec<_> = std::fs::read_dir(dir.path())
        .expect("list")
        .map(|item| item.expect("entry").file_name())
        .collect();
    assert_eq!(left, [OsString::from("state.json")]);
}

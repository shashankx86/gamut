use std::env;
use std::fs;
use std::io;
use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const APP_DIR_NAME: &str = "gamut";

pub(crate) fn app_config_path(file_name: &str) -> PathBuf {
    base_dir(env::var_os("XDG_CONFIG_HOME"), ".config")
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(APP_DIR_NAME)
        .join(file_name)
}

pub(crate) fn app_cache_path(file_name: &str) -> PathBuf {
    base_dir(env::var_os("XDG_CACHE_HOME"), ".cache")
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(APP_DIR_NAME)
        .join(file_name)
}

pub(crate) fn write_atomic(path: &Path, contents: &[u8]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let temp_path = temporary_path(path);
    let mut file = temporary_file(&temp_path)?;
    file.write_all(contents)?;
    file.sync_all()?;
    drop(file);

    fs::rename(&temp_path, path).or_else(|error| {
        let _ = fs::remove_file(&temp_path);
        Err(error)
    })
}

pub(crate) fn read_limited(path: &Path, max_bytes: u64) -> io::Result<Vec<u8>> {
    let metadata = fs::metadata(path)?;
    if metadata.len() > max_bytes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("file exceeds maximum size of {max_bytes} bytes"),
        ));
    }

    fs::read(path)
}

fn base_dir(primary: Option<std::ffi::OsString>, home_suffix: &str) -> Option<PathBuf> {
    if let Some(path) = primary {
        return Some(PathBuf::from(path));
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(home_suffix))
}

fn temporary_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let suffix = format!("{}.{}.tmp", std::process::id(), nanos);

    match path.file_name().and_then(|name| name.to_str()) {
        Some(file_name) => path.with_file_name(format!("{file_name}.{suffix}")),
        None => path.with_extension(suffix),
    }
}

fn temporary_file(path: &Path) -> io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);

    #[cfg(unix)]
    {
        options.mode(0o600);
    }

    options.open(path)
}

#[cfg(test)]
mod tests {
    use super::{app_cache_path, app_config_path, read_limited, write_atomic};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_path(file_name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();

        PathBuf::from("/tmp")
            .join(format!("gamut-storage-test-{nanos}"))
            .join(file_name)
    }

    #[test]
    fn app_paths_use_expected_directories() {
        assert!(app_config_path("preferences.toml").ends_with("gamut/preferences.toml"));
        assert!(app_cache_path("applications.bin").ends_with("gamut/applications.bin"));
    }

    #[test]
    fn atomic_write_replaces_existing_file_contents() {
        let path = unique_path("sample.bin");

        write_atomic(&path, b"first").expect("should write first payload");
        write_atomic(&path, b"second").expect("should replace payload");

        let contents = fs::read(&path).expect("should read replaced file");
        assert_eq!(contents, b"second");

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn limited_read_rejects_oversized_files() {
        let path = unique_path("oversized.bin");

        write_atomic(&path, b"abcdef").expect("should write file");

        let error = read_limited(&path, 3).expect_err("should reject oversized file");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

        let _ = fs::remove_file(&path);
        if let Some(parent) = path.parent() {
            let _ = fs::remove_dir_all(parent);
        }
    }
}

use std::env;
use std::path::{Path, PathBuf};

const SOCKET_NAME: &str = "gamut-launcher.sock";
const APP_RUNTIME_DIR_NAME: &str = "gamut";

pub fn socket_path() -> PathBuf {
    let runtime_dir = env::var_os("XDG_RUNTIME_DIR").map(PathBuf::from);
    let uid = current_uid();
    let run_user_dir = PathBuf::from(format!("/run/user/{uid}"));
    let state_home = state_home_dir();

    socket_path_for(
        runtime_dir.as_deref(),
        uid,
        run_user_dir.is_dir(),
        state_home.as_deref(),
    )
}

fn state_home_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("XDG_STATE_HOME").map(PathBuf::from) {
        return Some(path);
    }

    env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(".local/state"))
}

fn socket_path_for(
    runtime_dir: Option<&Path>,
    uid: u32,
    run_user_exists: bool,
    state_home: Option<&Path>,
) -> PathBuf {
    if let Some(dir) = runtime_dir {
        return dir.join(SOCKET_NAME);
    }

    if run_user_exists {
        return PathBuf::from(format!("/run/user/{uid}/{SOCKET_NAME}"));
    }

    if let Some(state_home) = state_home {
        return state_home.join(APP_RUNTIME_DIR_NAME).join(SOCKET_NAME);
    }

    PathBuf::from(format!("/tmp/{SOCKET_NAME}.{uid}"))
}

fn current_uid() -> u32 {
    // SAFETY: `geteuid` has no preconditions and cannot fail.
    unsafe { libc::geteuid() }
}

#[cfg(test)]
mod tests {
    use super::{SOCKET_NAME, socket_path_for};
    use std::path::PathBuf;

    #[test]
    fn prefers_xdg_runtime_socket_path() {
        let path = socket_path_for(
            Some(PathBuf::from("/tmp/runtime").as_path()),
            1000,
            true,
            Some(PathBuf::from("/state").as_path()),
        );

        assert_eq!(path, PathBuf::from(format!("/tmp/runtime/{SOCKET_NAME}")));
    }

    #[test]
    fn falls_back_to_run_user_socket_path() {
        let path = socket_path_for(None, 1001, true, Some(PathBuf::from("/state").as_path()));
        assert_eq!(path, PathBuf::from(format!("/run/user/1001/{SOCKET_NAME}")));
    }

    #[test]
    fn falls_back_to_state_home_socket_path() {
        let path = socket_path_for(None, 1002, false, Some(PathBuf::from("/state").as_path()));
        assert_eq!(path, PathBuf::from(format!("/state/gamut/{SOCKET_NAME}")));
    }

    #[test]
    fn falls_back_to_tmp_socket_path_as_last_resort() {
        let path = socket_path_for(None, 1003, false, None);
        assert_eq!(path, PathBuf::from(format!("/tmp/{SOCKET_NAME}.1003")));
    }
}

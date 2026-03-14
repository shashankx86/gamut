use std::env;
use std::process::{Command, Stdio};

const PREFERENCES_ARG: &str = "--preferences";

pub(crate) fn launch_preferences_app() -> Result<(), Box<dyn std::error::Error>> {
    let current_exe = env::current_exe()?;

    Command::new(&current_exe)
        .arg(PREFERENCES_ARG)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .spawn()
        .map(|_| ())
        .map_err(|error| {
            format!(
                "failed to launch preferences app {}: {error}",
                current_exe.display()
            )
            .into()
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn preferences_arg_matches_cli_flag() {
        assert_eq!(super::PREFERENCES_ARG, "--preferences");
    }
}

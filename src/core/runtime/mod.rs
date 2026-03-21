mod daemon;

use crate::core::cli::{CliCommand, CliMode, parse_command, print_help, print_version};
use crate::core::ipc::IpcCommand;
use crate::core::logging;
use crate::core::preferences::run_config;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::time::Duration;

use log::info;

type DynError = Box<dyn Error>;
const DAEMON_START_RETRIES: usize = 40;
const DAEMON_START_DELAY: Duration = Duration::from_millis(50);

pub fn run() -> Result<(), DynError> {
    run_with_args(env::args_os().skip(1))
}

fn run_with_args<I>(args: I) -> Result<(), DynError>
where
    I: IntoIterator<Item = OsString>,
{
    match parse_command(args) {
        Ok(CliCommand::Help(topic)) => {
            print_help(topic);
            Ok(())
        }
        Ok(CliCommand::Version) => {
            print_version();
            Ok(())
        }
        Ok(CliCommand::Config(command)) => run_config(command),
        Ok(CliCommand::Run(mode)) => {
            logging::init();
            info!("handling {} command", mode.name());
            run_mode(mode)
        }
        Err(error) => {
            print_help(error.help_topic());
            Err(error.message().to_string().into())
        }
    }
}

fn run_mode(mode: CliMode) -> Result<(), DynError> {
    match mode {
        CliMode::Daemon => daemon::run_daemon(),
        CliMode::Toggle => daemon::ensure_daemon_and_send(IpcCommand::Toggle {
            target_output: None,
        }),
        CliMode::Quit => daemon::send_quit(),
    }
}

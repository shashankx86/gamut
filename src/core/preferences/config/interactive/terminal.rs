use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::time::Duration;

pub(super) fn read_byte(stdin: &mut io::Stdin) -> io::Result<u8> {
    let mut buffer = [0_u8; 1];
    stdin.read_exact(&mut buffer)?;
    Ok(buffer[0])
}

pub(super) fn read_optional_byte(
    stdin: &mut io::Stdin,
    timeout: Duration,
) -> io::Result<Option<u8>> {
    if wait_for_stdin(stdin.as_raw_fd(), timeout)? {
        read_byte(stdin).map(Some)
    } else {
        Ok(None)
    }
}

pub(super) fn prompt_yes_no(label: &str, default: bool) -> io::Result<bool> {
    let suffix = if default { "[Y/n]" } else { "[y/N]" };

    loop {
        print!("{label} {suffix}: ");
        io::stdout().flush()?;

        let mut buffer = String::new();
        io::stdin().read_line(&mut buffer)?;

        match buffer.trim().to_ascii_lowercase().as_str() {
            "" => return Ok(default),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => eprintln!("Please answer yes or no."),
        }
    }
}

fn wait_for_stdin(fd: i32, timeout: Duration) -> io::Result<bool> {
    let mut readfds = unsafe {
        let mut set = std::mem::zeroed::<libc::fd_set>();
        libc::FD_ZERO(&mut set);
        libc::FD_SET(fd, &mut set);
        set
    };

    let mut timeval = libc::timeval {
        tv_sec: timeout.as_secs() as libc::time_t,
        tv_usec: timeout.subsec_micros() as libc::suseconds_t,
    };

    let ready = unsafe {
        libc::select(
            fd + 1,
            &mut readfds,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut timeval,
        )
    };

    if ready < 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(ready > 0)
}

pub(super) struct RawModeGuard {
    fd: i32,
    original: libc::termios,
}

impl RawModeGuard {
    pub(super) fn new(fd: i32) -> io::Result<Self> {
        let mut original = unsafe { std::mem::zeroed::<libc::termios>() };

        if unsafe { libc::tcgetattr(fd, &mut original) } != 0 {
            let error = io::Error::last_os_error();

            return if error.raw_os_error() == Some(libc::ENOTTY) {
                Err(io::Error::other(
                    "interactive shortcut capture requires a real terminal",
                ))
            } else {
                Err(error)
            };
        }

        let mut raw = original;
        raw.c_lflag &= !(libc::ICANON | libc::ECHO);
        raw.c_iflag &= !(libc::IXON | libc::ICRNL);
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;

        if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &raw) } != 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self { fd, original })
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        unsafe {
            libc::tcsetattr(self.fd, libc::TCSANOW, &self.original);
        }
    }
}

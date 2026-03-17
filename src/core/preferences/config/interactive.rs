use super::{ShortcutAction, ShortcutBinding, ShortcutPreferences};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::time::Duration;

const ESCAPE: u8 = 0x1b;
const CTRL_C: u8 = 0x03;
const ENTER: u8 = b'\n';
const CARRIAGE_RETURN: u8 = b'\r';
const TAB: u8 = b'\t';
const BACKSPACE: u8 = 0x7f;
const ESCAPE_SEQUENCE_WAIT: Duration = Duration::from_millis(40);

pub fn configure_shortcuts(
    shortcuts: &mut ShortcutPreferences,
    action: Option<ShortcutAction>,
) -> Result<bool, Box<dyn std::error::Error>> {
    match action {
        Some(action) => capture_and_apply(shortcuts, action),
        None => {
            let mut changed = false;

            for action in ShortcutAction::ALL {
                println!();
                println!("{}", action.label());
                println!("Current: {}", shortcuts.binding(action));
                println!("{}", action.description());

                if !prompt_yes_no("Capture a new shortcut", true)? {
                    continue;
                }

                changed |= capture_and_apply(shortcuts, action)?;
            }

            Ok(changed)
        }
    }
}

fn capture_and_apply(
    shortcuts: &mut ShortcutPreferences,
    action: ShortcutAction,
) -> Result<bool, Box<dyn std::error::Error>> {
    loop {
        println!();
        println!("{}", action.label());
        println!("Current: {}", shortcuts.binding(action));
        println!("Press the new key combo now.");
        println!("Ctrl+C aborts.");

        let binding = capture_binding()?;
        println!("Detected: {binding}");

        if prompt_yes_no("Save this shortcut", true)? {
            shortcuts.update_binding(action, binding)?;
            println!(
                "Saved {} = {}",
                action.config_key(),
                shortcuts.binding(action)
            );
            return Ok(true);
        }

        if !prompt_yes_no("Capture again", true)? {
            println!("Skipped {}.", action.config_key());
            return Ok(false);
        }
    }
}

fn capture_binding() -> Result<ShortcutBinding, Box<dyn std::error::Error>> {
    let mut stdin = io::stdin();
    let _guard = RawModeGuard::new(stdin.as_raw_fd())?;

    loop {
        let first = read_byte(&mut stdin)?;

        if first == CTRL_C {
            return Err("shortcut capture cancelled".into());
        }

        if let Some(binding) = parse_binding(first, &mut stdin)? {
            println!();
            return Ok(binding);
        }
    }
}

fn parse_binding(
    first: u8,
    stdin: &mut io::Stdin,
) -> Result<Option<ShortcutBinding>, Box<dyn std::error::Error>> {
    let binding = match first {
        ESCAPE => parse_escape_sequence(stdin)?,
        ENTER | CARRIAGE_RETURN => ShortcutBinding::named("Enter"),
        TAB => ShortcutBinding::named("Tab"),
        BACKSPACE | 0x08 => ShortcutBinding::named("Backspace"),
        0x01..=0x1a => ctrl_alpha_binding(first),
        byte if byte.is_ascii_uppercase() => ShortcutBinding {
            ctrl: false,
            alt: false,
            shift: true,
            super_key: false,
            key: (byte as char).to_ascii_lowercase().to_string(),
        },
        byte if byte.is_ascii_lowercase() || byte.is_ascii_digit() || is_symbol(byte) => {
            ShortcutBinding {
                ctrl: false,
                alt: false,
                shift: false,
                super_key: false,
                key: (byte as char).to_string(),
            }
        }
        _ => return Ok(None),
    };

    Ok(Some(binding))
}

fn ctrl_alpha_binding(byte: u8) -> ShortcutBinding {
    let key = ((byte - 1) + b'a') as char;

    ShortcutBinding {
        ctrl: true,
        alt: false,
        shift: false,
        super_key: false,
        key: key.to_string(),
    }
}

fn parse_escape_sequence(
    stdin: &mut io::Stdin,
) -> Result<ShortcutBinding, Box<dyn std::error::Error>> {
    let Some(next) = read_optional_byte(stdin, ESCAPE_SEQUENCE_WAIT)? else {
        return Ok(ShortcutBinding::named("Escape"));
    };

    if next == b'[' {
        return parse_csi_sequence(stdin);
    }

    if next == b'O' {
        return parse_ss3_sequence(stdin);
    }

    if next == CTRL_C {
        return Err("shortcut capture cancelled".into());
    }

    let mut binding = parse_alt_modified(next);
    binding.alt = true;
    Ok(binding)
}

fn parse_ss3_sequence(
    stdin: &mut io::Stdin,
) -> Result<ShortcutBinding, Box<dyn std::error::Error>> {
    let final_byte = read_byte(stdin)?;

    let key = match final_byte {
        b'P' => "F1",
        b'Q' => "F2",
        b'R' => "F3",
        b'S' => "F4",
        _ => "Escape",
    };

    Ok(ShortcutBinding::named(key))
}

fn parse_csi_sequence(
    stdin: &mut io::Stdin,
) -> Result<ShortcutBinding, Box<dyn std::error::Error>> {
    let mut sequence = Vec::new();

    loop {
        let byte = read_byte(stdin)?;
        sequence.push(byte);

        if is_csi_final(byte) {
            break;
        }
    }

    let final_byte = *sequence.last().ok_or("invalid escape sequence")?;
    let params = String::from_utf8_lossy(&sequence[..sequence.len().saturating_sub(1)]);
    let codes: Vec<u8> = params
        .split(';')
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse::<u8>().ok())
        .collect();

    let mut binding = ShortcutBinding::named(csi_key_name(final_byte, codes.first().copied()));

    if let Some(modifier_code) = codes.get(1).copied().or_else(|| {
        if matches!(final_byte, b'A' | b'B' | b'C' | b'D' | b'H' | b'F') {
            codes.first().copied().filter(|code| *code > 1)
        } else {
            None
        }
    }) {
        apply_csi_modifiers(&mut binding, modifier_code);
    }

    Ok(binding)
}

fn csi_key_name(final_byte: u8, first_code: Option<u8>) -> &'static str {
    match final_byte {
        b'A' => "ArrowUp",
        b'B' => "ArrowDown",
        b'C' => "ArrowRight",
        b'D' => "ArrowLeft",
        b'F' => "End",
        b'H' => "Home",
        b'~' => match first_code.unwrap_or_default() {
            1 | 7 => "Home",
            2 => "Insert",
            3 => "Delete",
            4 | 8 => "End",
            5 => "PageUp",
            6 => "PageDown",
            11 => "F1",
            12 => "F2",
            13 => "F3",
            14 => "F4",
            15 => "F5",
            17 => "F6",
            18 => "F7",
            19 => "F8",
            20 => "F9",
            21 => "F10",
            23 => "F11",
            24 => "F12",
            _ => "Escape",
        },
        _ => "Escape",
    }
}

fn apply_csi_modifiers(binding: &mut ShortcutBinding, code: u8) {
    match code {
        2 => binding.shift = true,
        3 => binding.alt = true,
        4 => {
            binding.shift = true;
            binding.alt = true;
        }
        5 => binding.ctrl = true,
        6 => {
            binding.shift = true;
            binding.ctrl = true;
        }
        7 => {
            binding.alt = true;
            binding.ctrl = true;
        }
        8 => {
            binding.shift = true;
            binding.alt = true;
            binding.ctrl = true;
        }
        _ => {}
    }
}

fn parse_alt_modified(byte: u8) -> ShortcutBinding {
    let mut binding = match byte {
        ENTER | CARRIAGE_RETURN => ShortcutBinding::named("Enter"),
        TAB => ShortcutBinding::named("Tab"),
        BACKSPACE | 0x08 => ShortcutBinding::named("Backspace"),
        0x01..=0x1a => ctrl_alpha_binding(byte),
        value if value.is_ascii_uppercase() => ShortcutBinding {
            ctrl: false,
            alt: false,
            shift: true,
            super_key: false,
            key: (value as char).to_ascii_lowercase().to_string(),
        },
        value => ShortcutBinding {
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
            key: (value as char).to_string(),
        },
    };

    binding.alt = true;
    binding
}

fn is_symbol(byte: u8) -> bool {
    matches!(
        byte,
        b'/' | b'\\' | b'.' | b',' | b';' | b'\'' | b'[' | b']' | b'-' | b'=' | b'`'
    )
}

fn is_csi_final(byte: u8) -> bool {
    (0x40..=0x7e).contains(&byte)
}

fn read_byte(stdin: &mut io::Stdin) -> io::Result<u8> {
    let mut buffer = [0_u8; 1];
    stdin.read_exact(&mut buffer)?;
    Ok(buffer[0])
}

fn read_optional_byte(stdin: &mut io::Stdin, timeout: Duration) -> io::Result<Option<u8>> {
    if wait_for_stdin(stdin.as_raw_fd(), timeout)? {
        read_byte(stdin).map(Some)
    } else {
        Ok(None)
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

fn prompt_yes_no(label: &str, default: bool) -> io::Result<bool> {
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

struct RawModeGuard {
    fd: i32,
    original: libc::termios,
}

impl RawModeGuard {
    fn new(fd: i32) -> io::Result<Self> {
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

#[cfg(test)]
mod tests {
    use super::{apply_csi_modifiers, csi_key_name, ctrl_alpha_binding, parse_alt_modified};

    #[test]
    fn ctrl_alpha_binding_maps_ctrl_k() {
        let binding = ctrl_alpha_binding(11);

        assert!(binding.ctrl);
        assert_eq!(binding.key, "k");
    }

    #[test]
    fn alt_modified_uppercase_preserves_shift() {
        let binding = parse_alt_modified(b'K');

        assert!(binding.alt);
        assert!(binding.shift);
        assert_eq!(binding.key, "k");
    }

    #[test]
    fn csi_modifier_code_sets_ctrl_shift() {
        let mut binding = super::ShortcutBinding::named("ArrowUp");
        apply_csi_modifiers(&mut binding, 6);

        assert!(binding.ctrl);
        assert!(binding.shift);
    }

    #[test]
    fn csi_key_name_maps_arrow_keys() {
        assert_eq!(csi_key_name(b'A', None), "ArrowUp");
        assert_eq!(csi_key_name(b'~', Some(5)), "PageUp");
    }
}

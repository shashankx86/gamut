mod keys;
mod terminal;

use crate::core::preferences::{ShortcutAction, ShortcutBinding, ShortcutPreferences};
use keys::{
    BACKSPACE, CARRIAGE_RETURN, CTRL_C, ENTER, ESCAPE, TAB, apply_csi_modifiers, csi_key_code,
    ctrl_alpha_binding, is_csi_final, is_symbol, parse_alt_modified,
};
use std::io;
use std::os::fd::AsRawFd;
use std::time::Duration;
use terminal::{RawModeGuard, prompt_yes_no, read_byte, read_optional_byte};

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
        ENTER | CARRIAGE_RETURN => {
            ShortcutBinding::from_key_code(crate::core::preferences::VK_ENTER)
        }
        TAB => ShortcutBinding::from_key_code(crate::core::preferences::VK_TAB),
        BACKSPACE | 0x08 => ShortcutBinding::from_key_code(crate::core::preferences::VK_BACKSPACE),
        0x01..=0x1a => ctrl_alpha_binding(first),
        byte if byte.is_ascii_uppercase() => ShortcutBinding {
            ctrl: false,
            alt: false,
            shift: true,
            super_key: false,
            key_codes: vec![byte as u32],
        },
        byte if byte.is_ascii_lowercase() || byte.is_ascii_digit() || is_symbol(byte) => {
            ShortcutBinding {
                ctrl: false,
                alt: false,
                shift: false,
                super_key: false,
                key_codes: vec![
                    crate::core::preferences::virtual_keycode_from_ascii_char(byte as char)
                        .ok_or("unsupported key")?,
                ],
            }
        }
        _ => return Ok(None),
    };

    Ok(Some(binding))
}

fn parse_escape_sequence(
    stdin: &mut io::Stdin,
) -> Result<ShortcutBinding, Box<dyn std::error::Error>> {
    let Some(next) = read_optional_byte(stdin, ESCAPE_SEQUENCE_WAIT)? else {
        return Ok(ShortcutBinding::from_key_code(
            crate::core::preferences::VK_ESCAPE,
        ));
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
        b'P' => crate::core::preferences::VK_F1,
        b'Q' => crate::core::preferences::VK_F1 + 1,
        b'R' => crate::core::preferences::VK_F1 + 2,
        b'S' => crate::core::preferences::VK_F1 + 3,
        _ => crate::core::preferences::VK_ESCAPE,
    };

    Ok(ShortcutBinding::from_key_code(key))
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

    let mut binding =
        ShortcutBinding::from_key_code(csi_key_code(final_byte, codes.first().copied()));

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

#[cfg(test)]
mod tests {
    use super::{apply_csi_modifiers, csi_key_code, ctrl_alpha_binding, parse_alt_modified};
    use crate::core::preferences::ShortcutBinding;

    #[test]
    fn ctrl_alpha_binding_maps_ctrl_k() {
        let binding = ctrl_alpha_binding(11);

        assert!(binding.ctrl);
        assert_eq!(binding.key_codes, vec![75]);
    }

    #[test]
    fn alt_modified_uppercase_preserves_shift() {
        let binding = parse_alt_modified(b'K');

        assert!(binding.alt);
        assert!(binding.shift);
        assert_eq!(binding.key_codes, vec![75]);
    }

    #[test]
    fn csi_modifier_code_sets_ctrl_shift() {
        let mut binding = ShortcutBinding::from_key_code(crate::core::preferences::VK_UP);
        apply_csi_modifiers(&mut binding, 6);

        assert!(binding.ctrl);
        assert!(binding.shift);
    }

    #[test]
    fn csi_key_name_maps_arrow_keys() {
        assert_eq!(csi_key_code(b'A', None), crate::core::preferences::VK_UP);
        assert_eq!(
            csi_key_code(b'~', Some(5)),
            crate::core::preferences::VK_PAGE_UP
        );
    }
}

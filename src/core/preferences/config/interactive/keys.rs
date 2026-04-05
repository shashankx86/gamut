use crate::core::preferences::ShortcutBinding;

pub(super) const ESCAPE: u8 = 0x1b;
pub(super) const CTRL_C: u8 = 0x03;
pub(super) const ENTER: u8 = b'\n';
pub(super) const CARRIAGE_RETURN: u8 = b'\r';
pub(super) const TAB: u8 = b'\t';
pub(super) const BACKSPACE: u8 = 0x7f;

pub(super) fn ctrl_alpha_binding(byte: u8) -> ShortcutBinding {
    let key = ((byte - 1) + b'a') as char;

    ShortcutBinding {
        ctrl: true,
        alt: false,
        shift: false,
        super_key: false,
        key: key.to_string(),
    }
}

pub(super) fn csi_key_name(final_byte: u8, first_code: Option<u8>) -> &'static str {
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

pub(super) fn apply_csi_modifiers(binding: &mut ShortcutBinding, code: u8) {
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

pub(super) fn parse_alt_modified(byte: u8) -> ShortcutBinding {
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

pub(super) fn is_symbol(byte: u8) -> bool {
    matches!(
        byte,
        b'/' | b'\\' | b'.' | b',' | b';' | b'\'' | b'[' | b']' | b'-' | b'=' | b'`'
    )
}

pub(super) fn is_csi_final(byte: u8) -> bool {
    (0x40..=0x7e).contains(&byte)
}

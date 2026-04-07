use crate::core::preferences::{
    ShortcutBinding, VK_BACKSPACE, VK_DELETE, VK_DOWN, VK_END, VK_ENTER, VK_ESCAPE, VK_F1, VK_HOME,
    VK_INSERT, VK_LEFT, VK_PAGE_DOWN, VK_PAGE_UP, VK_RIGHT, VK_TAB, VK_UP,
    virtual_keycode_from_ascii_char,
};

pub(super) const ESCAPE: u8 = 0x1b;
pub(super) const CTRL_C: u8 = 0x03;
pub(super) const ENTER: u8 = b'\n';
pub(super) const CARRIAGE_RETURN: u8 = b'\r';
pub(super) const TAB: u8 = b'\t';
pub(super) const BACKSPACE: u8 = 0x7f;

pub(super) fn ctrl_alpha_binding(byte: u8) -> ShortcutBinding {
    let key = ((byte - 1) + b'A') as u32;

    ShortcutBinding {
        ctrl: true,
        alt: false,
        shift: false,
        super_key: false,
        key_codes: vec![key],
    }
}

pub(super) fn csi_key_code(final_byte: u8, first_code: Option<u8>) -> u32 {
    match final_byte {
        b'A' => VK_UP,
        b'B' => VK_DOWN,
        b'C' => VK_RIGHT,
        b'D' => VK_LEFT,
        b'F' => VK_END,
        b'H' => VK_HOME,
        b'~' => match first_code.unwrap_or_default() {
            1 | 7 => VK_HOME,
            2 => VK_INSERT,
            3 => VK_DELETE,
            4 | 8 => VK_END,
            5 => VK_PAGE_UP,
            6 => VK_PAGE_DOWN,
            11 => VK_F1,
            12 => VK_F1 + 1,
            13 => VK_F1 + 2,
            14 => VK_F1 + 3,
            15 => VK_F1 + 4,
            17 => VK_F1 + 5,
            18 => VK_F1 + 6,
            19 => VK_F1 + 7,
            20 => VK_F1 + 8,
            21 => VK_F1 + 9,
            23 => VK_F1 + 10,
            24 => VK_F1 + 11,
            _ => VK_ESCAPE,
        },
        _ => VK_ESCAPE,
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
        ENTER | CARRIAGE_RETURN => ShortcutBinding::from_key_code(VK_ENTER),
        TAB => ShortcutBinding::from_key_code(VK_TAB),
        BACKSPACE | 0x08 => ShortcutBinding::from_key_code(VK_BACKSPACE),
        0x01..=0x1a => ctrl_alpha_binding(byte),
        value if value.is_ascii_uppercase() => ShortcutBinding {
            ctrl: false,
            alt: false,
            shift: true,
            super_key: false,
            key_codes: vec![value as u32],
        },
        value => ShortcutBinding {
            ctrl: false,
            alt: false,
            shift: false,
            super_key: false,
            key_codes: vec![virtual_keycode_from_ascii_char(value as char).unwrap_or(VK_ESCAPE)],
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

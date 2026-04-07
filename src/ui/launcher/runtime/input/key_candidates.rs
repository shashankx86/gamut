use iced::keyboard::{self, Key};

pub(in crate::ui::launcher) fn pressed_keycode_candidates(
    key: &Key,
    physical_key: keyboard::key::Physical,
) -> Vec<u32> {
    let mut candidates = Vec::new();

    match key.as_ref() {
        Key::Named(named) => push_candidate(&mut candidates, &format!("{named:?}")),
        Key::Character(value) => push_candidate(&mut candidates, &value.to_string()),
        Key::Unidentified => {}
    }

    if let Some(value) = key.to_latin(physical_key) {
        push_candidate(&mut candidates, &value.to_string());
    }

    match physical_key {
        keyboard::key::Physical::Code(code) => {
            push_candidate(&mut candidates, &format!("{code:?}"))
        }
        keyboard::key::Physical::Unidentified(native) => {
            push_candidate(&mut candidates, &format!("{native:?}"));
            push_candidate(&mut candidates, &format!("{physical_key:?}"));
        }
    }

    candidates
}

fn push_candidate(candidates: &mut Vec<u32>, value: &str) {
    if let Some(code) = crate::core::preferences::virtual_keycode_from_token(value)
        && !candidates.contains(&code)
    {
        candidates.push(code);
    }
}

pub(in crate::ui::launcher) fn matches_alt_action_key(
    key: &Key,
    physical_key: keyboard::key::Physical,
    digit: char,
) -> bool {
    let Some(number) = digit.to_digit(10) else {
        return false;
    };

    let ascii_code = digit as u32;
    let numpad_code = crate::core::preferences::VK_NUMPAD_0 + number;

    pressed_keycode_candidates(key, physical_key)
        .into_iter()
        .any(|pressed| pressed == ascii_code || pressed == numpad_code)
}

use iced::keyboard::{self, Key};

pub(in crate::ui::launcher) fn pressed_key_candidates(
    key: &Key,
    physical_key: keyboard::key::Physical,
) -> Vec<String> {
    let mut candidates = Vec::new();

    match key.as_ref() {
        Key::Named(named) => push_candidate(&mut candidates, format!("{named:?}")),
        Key::Character(value) => push_candidate(&mut candidates, value.to_string()),
        Key::Unidentified => {}
    }

    if let Some(value) = key.to_latin(physical_key) {
        push_candidate(&mut candidates, value.to_string());
    }

    match physical_key {
        keyboard::key::Physical::Code(code) => push_candidate(&mut candidates, format!("{code:?}")),
        keyboard::key::Physical::Unidentified(native) => {
            push_candidate(&mut candidates, format!("{native:?}"));
            push_candidate(&mut candidates, format!("{physical_key:?}"));
        }
    }

    candidates
}

fn push_candidate(candidates: &mut Vec<String>, value: String) {
    let normalized = normalize_binding_key(&value);

    if !normalized.is_empty() && !candidates.contains(&normalized) {
        candidates.push(normalized);
    }
}

pub(in crate::ui::launcher) fn normalize_binding_key(value: &str) -> String {
    crate::core::preferences::normalize_key_token(value)
}

pub(in crate::ui::launcher) fn matches_alt_action_key(
    key: &Key,
    physical_key: keyboard::key::Physical,
    digit: char,
) -> bool {
    let needle = digit.to_string();
    let digit_code = format!("digit{digit}");
    let numpad_code = format!("numpad{digit}");

    pressed_key_candidates(key, physical_key)
        .into_iter()
        .any(|pressed| pressed == needle || pressed == digit_code || pressed == numpad_code)
}

use std::collections::HashSet;

pub(super) fn context_icon_candidates(
    categories: &[String],
    name: &str,
    exec_line: &str,
) -> Vec<&'static str> {
    let mut names = Vec::new();

    let has_category = |wanted: &str| {
        categories
            .iter()
            .any(|category| category.eq_ignore_ascii_case(wanted))
    };

    if has_category("Printing") {
        names.extend(["printer", "printer-network"]);
    }
    if has_category("Scanner") {
        names.extend(["scanner", "scanner-photo"]);
    }
    if has_category("Settings") || has_category("System") || has_category("HardwareSettings") {
        names.extend(["preferences-system", "applications-system"]);
    }
    if has_category("Network") {
        names.extend(["network-workgroup", "applications-internet"]);
    }
    if has_category("Office") {
        names.push("applications-office");
    }
    if has_category("Graphics") {
        names.push("applications-graphics");
    }
    if has_category("AudioVideo") {
        names.push("applications-multimedia");
    }
    if has_category("Development") {
        names.push("applications-development");
    }
    if has_category("Utility") {
        names.push("applications-utilities");
    }

    let low_name = name.to_lowercase();
    let low_exec = exec_line.to_lowercase();
    if low_name.contains("printer")
        || low_exec.contains("printer")
        || low_name.contains("hplip")
        || low_exec.contains("hplip")
        || low_name.contains("hp device")
        || low_exec.contains("hp-")
    {
        names.extend(["printer", "printer-network"]);
    }

    dedupe_names(names)
}

fn dedupe_names(values: Vec<&'static str>) -> Vec<&'static str> {
    let mut seen = HashSet::new();

    values
        .into_iter()
        .filter(|value| seen.insert(*value))
        .collect()
}

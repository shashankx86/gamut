use super::HelpTopic;

const ASCII_ART: &str = r"                                                                                                             
                            GGGGGGGGG                                       GGGG                             
                            GGGGGGGGG                                       GGGG                             
                            GGGGGGGGG                                       GGGG                             
                            GGGG                                            GGGG                             
                            GGGGGGGGG GGGGGGGGG  GGGGGGGGGGGGGG HGGG  GGG GGGGGGG                            
                            GGGGGGGGG GGGGGGGGG  GGGGGGGGGGGGGG GGGG  GGGHGGGGGGG                            
                            GGGGGGGGG GGGGGGGGG  GGGGGGGGGGGGGG GGGG  GGGHGGGGGGG                            
                            GGGG GGGG GGGG  GGG  GGG  GGGG  GGG GGGG  GGGH  GGGG                             
                            GGGGGGGGG GGGGGGGGG  GGG  GGGG  GGG GGGGGGGGGH  GGGGG                            
                            GGGGGGGGG GGGGGGGGG  GGG  GGGG  GGG GGGGGGGGGH  GGGGG                            
                                                                                                             ";

pub fn help_text() -> String {
    format!(
        "{ASCII_ART}\n\nUsage:\n  {name} [OPTIONS] [COMMAND]\n\nCommands:\n  config                Manage launcher configuration from the CLI\n  toggle                Toggle the launcher (default)\n  daemon                Run the daemon process\n  quit                  Ask the running daemon to quit\n  help [COMMAND]        Show help for a command\n\nOptions:\n  -h, --help            Show this help message\n  -v, --version         Show version\n\nRun `{name} help config` for configuration help.\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn config_help_text() -> String {
    format!(
        "Usage:\n  {name} config <SUBCOMMAND>\n\nSubcommands:\n  show                              Print the active configuration\n  path                              Print the configuration file path\n  keys                              List all supported config keys\n  get <key>                         Read a single config value\n  set <key> <value>                 Update a single config value\n  reset <key>|all                   Reset one key or the full config\n  shortcut <SUBCOMMAND>             Manage keyboard shortcuts\n\nExamples:\n  {name} config show\n  {name} config get appearance.theme\n  {name} config set appearance.theme orange\n  {name} config set appearance.theme.accent '#FF7A00'\n  {name} config reset all\n\nHelp:\n  {name} help config show\n  {name} help config get\n  {name} help config set\n  {name} help config reset\n  {name} help config shortcut\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn config_show_help_text() -> String {
    format!(
        "Usage:\n  {name} config show\n\nPrint the current effective configuration grouped by section.\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn config_path_help_text() -> String {
    format!(
        "Usage:\n  {name} config path\n\nPrint the path to the primary config file.\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn config_keys_help_text() -> String {
    format!(
        "Usage:\n  {name} config keys\n\nList all supported config keys grouped by section.\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn config_get_help_text() -> String {
    format!(
        "Usage:\n  {name} config get <key>\n\nRead one config value.\n\nExample:\n  {name} config get appearance.theme\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn config_set_help_text() -> String {
    format!(
        "Usage:\n  {name} config set <key> <value>\n\nUpdate one config value.\n\nExamples:\n  {name} config set appearance.theme dark\n  {name} config set appearance.theme orange\n  {name} config set appearance.theme.background '#FFF4E8'\n  {name} config set layout.custom_top_margin 96\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn config_reset_help_text() -> String {
    format!(
        "Usage:\n  {name} config reset <key>\n  {name} config reset all\n\nReset a single key or the full configuration back to defaults.\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn shortcut_help_text() -> String {
    format!(
        "Usage:\n  {name} config shortcut <SUBCOMMAND>\n\nSubcommands:\n  list                                      Show current shortcut bindings\n  set <action> <binding>                    Set a shortcut directly\n  interactive [action]                      Capture a shortcut from live key presses\n\nActions:\n  launch-selected\n  expand\n  move-down\n  move-up\n  close-launcher\n\nBinding format:\n  Use virtual keycode arrays, for example `[75]`, `[17, 75]`, or `[17, 18, 16, 75]`. Duplicate bindings are allowed.\n\nHelp:\n  {name} help config shortcut list\n  {name} help config shortcut set\n  {name} help config shortcut interactive\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn shortcut_list_help_text() -> String {
    format!(
        "Usage:\n  {name} config shortcut list\n\nShow all saved shortcut bindings.\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn shortcut_set_help_text() -> String {
    format!(
        "Usage:\n  {name} config shortcut set <action> <binding>\n\nSet one shortcut directly.\n\nExample:\n  {name} config shortcut set move-up [17,75]\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn shortcut_interactive_help_text() -> String {
    format!(
        "Usage:\n  {name} config shortcut interactive [action]\n\nCapture a shortcut by pressing the actual key combo in the terminal, then confirm it before saving.\n\nExamples:\n  {name} config shortcut interactive\n  {name} config shortcut interactive close-launcher\n",
        name = env!("CARGO_PKG_NAME"),
    )
}

pub fn version_text() -> String {
    format!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
}

pub fn print_help(topic: HelpTopic) {
    let text = match topic {
        HelpTopic::Root => help_text(),
        HelpTopic::Config => config_help_text(),
        HelpTopic::ConfigShow => config_show_help_text(),
        HelpTopic::ConfigPath => config_path_help_text(),
        HelpTopic::ConfigKeys => config_keys_help_text(),
        HelpTopic::ConfigGet => config_get_help_text(),
        HelpTopic::ConfigSet => config_set_help_text(),
        HelpTopic::ConfigReset => config_reset_help_text(),
        HelpTopic::ConfigShortcut => shortcut_help_text(),
        HelpTopic::ConfigShortcutList => shortcut_list_help_text(),
        HelpTopic::ConfigShortcutSet => shortcut_set_help_text(),
        HelpTopic::ConfigShortcutInteractive => shortcut_interactive_help_text(),
    };

    println!("{text}");
}

pub fn print_version() {
    println!("{}", version_text());
}

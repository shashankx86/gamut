mod command;
mod display;
mod interactive;
mod key;
mod parsing;
mod runtime;
mod value;

pub use command::{ConfigCommand, ConfigResetTarget, ShortcutConfigCommand};
pub use key::ConfigKey;
pub use runtime::execute;

#[cfg(test)]
mod tests {
    use super::display::trim_float;

    #[test]
    fn trim_float_compacts_trailing_zeros() {
        assert_eq!(trim_float(12.0), "12");
        assert_eq!(trim_float(12.5), "12.5");
        assert_eq!(trim_float(12.34), "12.34");
    }
}

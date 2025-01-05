use crate::app::settings::Settings;

// Config
// TODO make this constructible using a builder
pub struct Config {
    pub app_name: String,
    pub settings: Settings
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_name: String::from("Icarus Engine"),
            settings: Settings::default()
        }
    }
}
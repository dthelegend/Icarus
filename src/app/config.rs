use crate::app::settings::Settings;

/// App config is designed to be used to construct the AppManager
pub struct Config {
    pub app_name: String,
    pub settings: Settings,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            app_name: String::from("Icarus Engine"),
            settings: Settings::default(),
        }
    }
}
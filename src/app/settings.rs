pub struct Settings {
    pub render_size: (u32, u32),
    pub preferred_device: Option<(u32,u32)>
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            render_size: (1920, 1080),
            preferred_device: None,
        }
    }
}
mod error;

use sdl3_sys as sys;
use error::{get_error, GameResult};

pub struct GameBuilder {
    flags: u32,
}

impl Default for GameBuilder {
    fn default() -> Self { Self::new() }
}

impl GameBuilder {
    pub const fn new() -> GameBuilder {
        GameBuilder { flags: 0 }
    }

    const fn add_flag(mut self, flag: u32) -> GameBuilder {
        self.flags |= flag;
        self
    }

    pub const fn use_video(self) -> Self {
        self.add_flag(sys::SDL_INIT_VIDEO)
    }

    pub const fn use_audio(self) -> Self {
        self.add_flag(sys::SDL_INIT_AUDIO)
    }

    pub const fn use_joystick(self) -> Self {
        self.add_flag(sys::SDL_INIT_JOYSTICK)
    }

    pub const fn use_haptic(self) -> Self {
        self.add_flag(sys::SDL_INIT_HAPTIC)
    }

    pub const fn use_gamepad(self) -> Self {
        self.add_flag(sys::SDL_INIT_GAMEPAD)
    }

    pub const fn use_events(self) -> Self {
        self.add_flag(sys::SDL_INIT_EVENTS)
    }

    pub const fn use_sensor(self) -> Self {
        self.add_flag(sys::SDL_INIT_SENSOR)
    }

    pub const fn use_camera(self) -> Self {
        self.add_flag(sys::SDL_INIT_CAMERA)
    }

    pub fn init(self) -> GameResult<GameSession> {
        let initialised_correctly = unsafe { sys::SDL_Init(self.flags) };
        if initialised_correctly {
            Ok(GameSession)
        } else {
            Err(get_error())
        }
    }
}

struct GameSession;

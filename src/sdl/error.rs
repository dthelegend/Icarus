use std::ffi::CStr;
use crate::sys;

pub type SDLResult<T> = Result<T, String>;

pub fn get_error() -> String {
    unsafe { CStr::from_ptr(sys::SDL_GetError()) }.to_str().unwrap().to_string()
}


#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#![allow(improper_ctypes)]

#[cfg(feature = "main")]
mod sdl_main;

include!(concat!(env!("OUT_DIR"), "/sdl_bindings.rs"));

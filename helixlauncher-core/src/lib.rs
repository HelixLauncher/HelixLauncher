#![deny(rust_2018_idioms)]

pub mod auth;
pub mod config;
pub mod game;
pub mod instance;
pub mod launcher;
mod util;

#[no_mangle]
pub extern "C" fn test(i: std::ffi::c_int) -> std::ffi::c_int {
    i + 1
}

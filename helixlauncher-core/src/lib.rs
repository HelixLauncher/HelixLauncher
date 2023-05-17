#![deny(rust_2018_idioms)]
#![feature(let_chains)]

pub mod auth;
pub mod config;
pub mod launch;

#[no_mangle]
pub extern "C" fn test(i: std::ffi::c_int) -> std::ffi::c_int {
    i + 1
}

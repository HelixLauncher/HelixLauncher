#![deny(rust_2018_idioms)]
#![deny(unstable_features)]

pub mod addon;
pub mod auth;
pub mod config;
mod fsutil;
pub mod launch;

#[no_mangle]
pub extern "C" fn test(i: std::ffi::c_int) -> std::ffi::c_int {
    i + 1
}

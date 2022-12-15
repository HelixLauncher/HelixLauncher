pub mod auth;
pub mod launcher;
pub mod config;
pub mod instance;
pub mod game;

#[no_mangle]
pub extern "C" fn test(i: std::ffi::c_int) -> std::ffi::c_int {
    i + 1
}

pub mod auth;
pub mod config;
pub mod game;
pub mod instance;
pub mod launcher;

#[no_mangle]
pub extern "C" fn test(i: std::ffi::c_int) -> std::ffi::c_int {
    i + 1
}

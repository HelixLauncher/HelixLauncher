mod authentication;

#[cfg(test)]
mod test;

#[no_mangle]
pub extern "C" fn test(i: std::ffi::c_int) -> std::ffi::c_int {
    i + 1
}
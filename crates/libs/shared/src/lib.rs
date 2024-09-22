use libc::{c_char, size_t};

#[repr(C)]
pub struct ListPackagesResult {
    pub err: *const c_char,
    pub data: *const *const c_char,
    pub len: size_t,
}

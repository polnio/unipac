use std::ffi::CString;
use libc::{c_char, size_t};

#[repr(C)]
pub struct ListPackagesResult {
    pub err: *const c_char,
    pub data: *const *const c_char,
    pub len: size_t,
}

pub fn call_list_packages(list_packages: fn() -> Result<Vec<String>, String>) -> ListPackagesResult {
    let result = match list_packages() {
        Ok(result) => result,
        Err(err) => {
            let err = CString::new(err).unwrap_or(c"Unknown Error".into());
            let c_err = err.as_ptr();
            std::mem::forget(err);
            return ListPackagesResult {
                data: std::ptr::null(),
                len: 0,
                err: c_err,
            };
        }
    };

    let len = result.len();
    let c_result = result
        .into_iter()
        .filter_map(|p| {
            let pc = CString::new(p).ok()?;
            let ptr = pc.as_ptr();
            std::mem::forget(pc);
            Some(ptr)
        })
        .collect::<Vec<_>>();
    let data = c_result.as_ptr();
    std::mem::forget(c_result);

    ListPackagesResult {
        data,
        len,
        err: std::ptr::null(),
    }
}

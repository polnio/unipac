use std::ffi::CString;
use libc::{c_char, size_t};

#[repr(C)]
pub struct ListPackagesResult {
    pub err: *mut c_char,
    pub data: *mut *mut c_char,
    pub len: size_t,
}

impl Drop for ListPackagesResult {
    fn drop(&mut self) {
        unsafe {
            if !self.err.is_null() {
                let _err = CString::from_raw(self.err);
            }
            if !self.data.is_null() {
                let data = Vec::from_raw_parts(self.data, self.len, self.len);
                for p in data {
                    let _p = CString::from_raw(p);
                }
            }
        }
    }
}

pub fn call_list_packages(list_packages: fn() -> Result<Vec<String>, String>) -> ListPackagesResult {
    let result = match list_packages() {
        Ok(result) => result,
        Err(err) => {
            let err = CString::new(err).unwrap_or_else(|_| c"Unknown Error".to_owned()).into_raw();
            return ListPackagesResult {
                data: std::ptr::null_mut(),
                len: 0,
                err,
            };
        }
    };

    let c_result = result
        .into_iter()
        .filter_map(|p| Some(CString::new(p).ok()?.into_raw()))
        .collect::<Vec<_>>();
    let (data, len, _) = c_result.into_raw_parts();

    ListPackagesResult {
        data,
        len,
        err: std::ptr::null_mut(),
    }
}

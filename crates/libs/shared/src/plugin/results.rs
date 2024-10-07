use safer_ffi::prelude::*;

#[repr(C)]
#[derive_ReprC]
#[ffi_export]
pub struct ListPackagesResult {
    pub err: Option<char_p::Box>,
    pub data: Option<c_slice::Box<char_p::Box>>,
}

fn str_rs_to_c(s: String) -> char_p::Box {
    let s = if s.contains("\0") {
        s.replace("\0", "")
    } else {
        s
    };
    unsafe { s.try_into().unwrap_unchecked() }
}

pub fn call_list_packages(
    list_packages: fn() -> Result<Vec<String>, String>,
) -> ListPackagesResult {
    let result = match list_packages() {
        Ok(result) => result,
        Err(err) => {
            return ListPackagesResult {
                data: None,
                err: Some(str_rs_to_c(err)),
            };
        }
    };

    let result = result
        .into_iter()
        .map(str_rs_to_c)
        .collect::<Vec<_>>()
        .into_boxed_slice()
        .into();

    ListPackagesResult {
        data: Some(result),
        err: None,
    }
}

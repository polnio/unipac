use safer_ffi::prelude::*;

#[repr(C)]
#[derive_ReprC]
pub struct ListPackagesResult {
    pub err: Option<str::Box>,
    pub data: Option<c_slice::Box<str::Box>>,
}

pub fn call_list_packages(
    list_packages: fn() -> Result<Vec<String>, String>,
) -> ListPackagesResult {
    let result = match list_packages() {
        Ok(result) => result,
        Err(err) => {
            return ListPackagesResult {
                data: None,
                err: Some(err.as_str().into()),
            };
        }
    };

    let result = result
        .into_iter()
        .map(str::Box::from)
        .collect::<Vec<_>>()
        .into_boxed_slice()
        .into();

    ListPackagesResult {
        data: Some(result),
        err: None,
    }
}

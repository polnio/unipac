use std::ffi::CString;

fn list_packages() -> Result<Vec<String>, String> {
    Ok(vec!["dummy".into()])
}

#[no_mangle]
extern "C" fn ffi_list_packages() -> unipac_shared::ListPackagesResult {
    let result = match list_packages() {
        Ok(result) => result,
        Err(err) => {
            let err = CString::new(err).unwrap_or(c"Unknown Error".into());
            let c_err = err.as_ptr();
            std::mem::forget(err);
            return unipac_shared::ListPackagesResult {
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

    unipac_shared::ListPackagesResult {
        data,
        len,
        err: std::ptr::null(),
    }
}

static INSTALLED_PACKAGES: [&str; 1] = ["example-rs"];

fn list_packages() -> Result<Vec<String>, String> {
    Ok(INSTALLED_PACKAGES.into_iter().map(String::from).collect())
}

unipac_shared::export_plugin!();

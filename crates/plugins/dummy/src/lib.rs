fn list_packages() -> Result<Vec<String>, String> {
    Ok(vec!["dummy".into()])
}

unipac_shared::export_plugin!();

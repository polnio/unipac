use anyhow::Context as _;

pub fn list_updates() {
    super::fetch_multiple(|p| p.list_updates().context("Failed to list updates"));
}

use anyhow::Context as _;

pub fn list_packages() {
    super::fetch_multiple(|p| p.list_packages().context("Failed to list packages"));
}

use anyhow::Context as _;

pub fn search(query: &'static str) {
    super::fetch_multiple(move |p| p.search(query).context("Failed to search packages"));
}

use anyhow::Context as _;

pub fn info(pname: &'static str) {
    super::fetch_one(move |p| p.info(pname).context("Failed to fetch package info"));
}

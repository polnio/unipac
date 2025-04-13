use crate::Config;
use anyhow::Context as _;

pub fn search(config: Config, query: String) {
    super::fetch_multiple(config, move |p| {
        p.search(query.clone()).context("Failed to search packages")
    });
}

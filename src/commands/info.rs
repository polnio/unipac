use crate::Config;
use anyhow::Context as _;

pub fn info(config: Config, pname: String) {
    super::fetch_one(config, move |p| {
        p.info(pname.clone())
            .context("Failed to fetch package info")
    });
}

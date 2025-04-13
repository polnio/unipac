use crate::Config;
use anyhow::Context as _;

pub fn list_packages(config: Config) {
    super::fetch_multiple(config, |p| {
        p.list_packages().context("Failed to list packages")
    });
}

use anyhow::Context as _;

pub fn remove(pname: &'static str) {
    super::fetch_nothing(move |p| p.remove(pname).context("Failed to remove package"));
}

use anyhow::Context as _;
use dialoguer::Confirm;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub fn update() {
    let n = Arc::new(AtomicUsize::new(0));
    let nn = Arc::clone(&n);
    super::fetch_multiple(move |p| {
        let rs = p.list_updates().context("Failed to list updates");
        let len = rs.as_ref().map(Vec::len).unwrap_or_default();
        nn.fetch_add(len, Ordering::Relaxed);
        rs
    });

    if n.load(Ordering::Relaxed) == 0 {
        println!("No updates available");
        return;
    }

    let confirmed = Confirm::new()
        .with_prompt("Do you want to update these packages?")
        .default(true)
        .interact()
        .unwrap();

    if !confirmed {
        return;
    }

    super::fetch_nothing(|p| p.update().context("Failed to update packages"));
}

use std::borrow::Cow;

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

#[derive(Debug, Clone, Default)]
pub struct Spinners(MultiProgress);
#[derive(Debug, Clone)]
pub struct Spinner(ProgressBar);

impl Spinners {
    pub fn new() -> Self {
        Self(MultiProgress::new())
    }
    pub fn add(&self, name: impl Into<Cow<'static, str>>) -> Spinner {
        let pb = self.0.add(ProgressBar::new(100));
        pb.set_style(
            ProgressStyle::with_template("{msg:<32} [{bar:40}] {pos}%")
                .unwrap()
                .progress_chars("##-"),
        );
        pb.set_message(name);
        Spinner(pb)
    }
    pub fn clear(&self) -> std::io::Result<()> {
        self.0.clear()
    }
}

impl Spinner {
    pub fn set(&self, progress: u8) {
        self.0.set_position(progress as u64);
    }
    pub fn finish(&self) {
        self.0.finish();
    }
}

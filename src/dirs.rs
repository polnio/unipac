use directories::ProjectDirs;
use std::sync::LazyLock;

pub static PROJECT_DIRS: LazyLock<ProjectDirs> = LazyLock::new(|| {
    ProjectDirs::from("dev", "polnio", "unipac")
        .expect("No valid home directory path could be retrieved from the operating system")
});

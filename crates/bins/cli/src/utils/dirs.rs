use std::sync::LazyLock;

use directories::ProjectDirs;

pub static UNIPAC_DIR: LazyLock<ProjectDirs> = LazyLock::new(|| {
    let Some(dirs) = ProjectDirs::from("com", "unipac", "unipac") else {
        eprintln!("Could not find the home directory");
        std::process::exit(1);
    };
    dirs
});

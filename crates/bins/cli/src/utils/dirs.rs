use directories::ProjectDirs;
use once_cell::sync::Lazy;

pub static UNIPAC_DIR: Lazy<ProjectDirs> = Lazy::new(|| {
    let Some(dirs) = ProjectDirs::from("com", "unipac", "unipac") else {
        eprintln!("Could not find the home directory");
        std::process::exit(1);
    };
    dirs
});

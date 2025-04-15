use clap::Parser as _;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, clap::Parser)]
pub struct Args {
    #[arg(short, long)]
    pub config_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}
static mut ARGS: Option<Args> = None;
impl Args {
    pub fn init() {
        unsafe {
            ARGS = Some(Args::parse());
        }
    }
    pub fn get() -> &'static Args {
        unsafe { ARGS.as_ref().unwrap_unchecked() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, clap::Subcommand)]
pub enum Command {
    List,
    Search { query: String },
    Info { pname: String },
}

use clap::ArgAction;
use std::path::PathBuf;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq, Eq, clap::Parser)]
pub struct Args {
    #[arg(short, long)]
    pub config_path: Option<PathBuf>,

    #[arg(
        long,
        action = ArgAction::Set,
        num_args = 0..=1,
        default_value_t = true,
    )]
    pub colors: bool,

    #[command(subcommand)]
    pub command: Command,

    #[arg(short, long, value_delimiter = ',')]
    pub plugins: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, clap::Subcommand)]
pub enum Command {
    List {
        #[arg(short, long)]
        updates: bool,
    },
    Search {
        query: String,
    },
    Info {
        pname: String,
    },
    Install {
        pname: String,
    },
    Remove {
        pname: String,
    },
    Update {
        #[arg(short, long)]
        list: bool,
    },
}

static ARGS: OnceLock<Args> = OnceLock::new();
impl Args {
    pub fn init(this: Self) {
        let _ = ARGS.set(this);
    }
    pub fn get() -> &'static Args {
        unsafe { ARGS.get().unwrap_unchecked() }
    }
    pub fn parse() -> Self {
        clap::Parser::parse()
    }
}

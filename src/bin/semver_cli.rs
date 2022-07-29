use clap::Parser;
use git2::Oid;
use log::LevelFilter;
use semver_calc::{error::SemVerError, history::History};

/// Calcualte Semantic Version from git history
#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// Path to the git repository
    #[clap(value_parser, value_name = "Path to Repository")]
    path: String,

    /// Commit hash to start from
    #[clap(short, long, value_parser, value_name = "Commit hash")]
    commit: Option<String>,

    /// Previous version
    ///
    /// Previous version to start version calculation from. Can be in the following formats:
    /// v1.0.2
    /// v1.0.2-pre.4
    /// v1.0.2-alpha.2
    #[clap(short, long, value_parser, value_name = "Version")]
    start_version: Option<String>,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

fn main() -> Result<(), SemVerError> {
    let cli = Args::parse();
    let verbosity = cli.verbose.log_level_filter();

    let mut builder = env_logger::builder();
    let logger = builder.filter_level(verbosity).format_target(false);

    if verbosity >= LevelFilter::Debug {
        logger.format_target(true);
    }

    match logger.try_init() {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{:?}", err)
        }
    }
    log::info!("Informational logging is active.");
    log::debug!("Debug logging is active.");
    log::trace!("Trace logging is active.");
    // todo since implementieren
    let commit = match cli.commit {
        Some(c) => Some(Oid::from_str(&c)?),
        None => None,
    };
    let semantic = History::analyze(cli.path, commit, cli.start_version)?;
    println!("{}", semantic.to_string());
    Ok(())
}

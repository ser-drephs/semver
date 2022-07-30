use clap::{ArgGroup, Parser};
use git2::Oid;
use log::LevelFilter;
use semver_calc::{
    error::SemVerError,
    history::{Analyser, CommitAnalyserPoint, HistoryAnalyser, TagAnalyserPoint},
    semantic::Semantic,
};

/// Calcualte semantic version from git history
#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[clap(group(
    ArgGroup::new("manual_input")
        .required(false)
        .multiple(true)
        .args(&["commit","previous-version"])
        .conflicts_with("tag")
))]
struct Args {
    /// Path to the git repository
    ///
    /// Path to the git repository or worktree.
    #[clap(value_parser, value_name = "Path to Repository")]
    path: String,

    /// Commit hash to start from
    ///
    /// Enter the commit hash which is the starting point for analysing the history.
    #[clap(short, long, value_parser, value_name = "Commit hash")]
    commit: Option<String>,

    /// Previous version
    ///
    /// Previous version to start version calculation from. Can be in the following formats:
    /// v1.0.2
    /// v1.0.2-pre.4
    /// v1.0.2-alpha.2
    #[clap(short, long, value_parser, value_name = "Version")]
    previous_version: Option<String>,

    /// Tag identifier
    ///
    /// The tag identifier will be used to get the starting commit and previos version information.
    #[clap(short, long, value_parser)]
    tag: Option<String>,

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
    // todo tag implementieren
    let commit = match cli.commit {
        Some(c) => Some(Oid::from_str(&c)?),
        None => None,
    };

    let semantic = match cli.tag { // todo logik auslagern
        Some(tag_name) => {
            let repository = HistoryAnalyser::get_repository(&cli.path)?;
            let point = TagAnalyserPoint::new(Some(&tag_name), &repository)?;
            HistoryAnalyser::run(cli.path, point)?
        }
        None => {
            let commit_point = CommitAnalyserPoint {
                since: commit,
                version_identifier: cli.previous_version,
            };
            HistoryAnalyser::run(cli.path, commit_point)?
        }
    };

    println!("{}", semantic.to_string());
    Ok(())
}

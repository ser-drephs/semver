use std::path::Path;

use git2::{Oid, Repository, Tag, Worktree};

use crate::{error::SemVerError, semantic::Semantic};

#[derive(Default)]
pub struct CommitAnalyserPoint {
    pub since: Option<Oid>,
    pub version_identifier: Option<String>,
}

pub struct TagAnalyserPoint {
    pub tag: Option<String>,
    inner: Option<CommitAnalyserPoint>,
}

pub trait AnalyserPoint {
    fn since(&self) -> Option<Oid>;

    fn version_identifier(&self) -> Option<String>;
}

impl AnalyserPoint for CommitAnalyserPoint {
    fn since(&self) -> Option<Oid> {
        self.since
    }

    fn version_identifier(&self) -> Option<String> {
        self.version_identifier
            .as_ref()
            .map(|version_identifier| version_identifier.to_owned())
    }
}

impl AnalyserPoint for TagAnalyserPoint {
    fn since(&self) -> Option<Oid> {
        match &self.inner {
            Some(inner) => inner.since.to_owned(),
            None => None,
        }
        // self.tag.as_ref().map(|tag| tag.id())
    }

    fn version_identifier(&self) -> Option<String> {
        match &self.inner {
            Some(inner) => inner.version_identifier.to_owned(),
            None => None,
        }
    }
}

impl TagAnalyserPoint {
    pub fn new(tag_name: Option<&str>, repository: &Repository) -> Result<Self, SemVerError> {
        match tag_name {
            Some(tag_name) => {
                let reference = repository
                    .find_reference(&format!("refs/tags/{}", tag_name))
                    .unwrap();
                if !reference.is_tag() {
                   return Err(SemVerError::SemanticError {
                        message: format!("{:?} is not a valid tag name", tag_name),
                    })
                }

                let commit = reference.peel_to_commit().unwrap();
                log::debug!("Tag {:?} is at commit {:?}", tag_name, commit.id());

                Ok(TagAnalyserPoint {
                    tag: Some(tag_name.to_owned()),
                    inner: Some(CommitAnalyserPoint {
                        since: Some(commit.id()),
                        version_identifier: Some(tag_name.to_owned()),
                    }),
                })
            }
            None => Ok(TagAnalyserPoint {
                tag: tag_name.map(|f| f.to_owned()),
                inner: Some(CommitAnalyserPoint {
                    ..Default::default()
                }),
            }),
        }
    }
}

pub trait Analyser {
    fn run<P: AsRef<Path>, A: AnalyserPoint>(path: P, point: A) -> Result<Semantic, SemVerError>;

    fn get_repository<P: AsRef<Path> + std::fmt::Debug>(
        path: P,
    ) -> Result<Repository, SemVerError> {
        match Repository::open(&path) {
            Ok(repository) => {
                if repository.is_worktree() {
                    log::info!(
                        "Provided repository is a worktree. Try conversion finding repository."
                    );
                    // panic!("worktrees are not supported!") // Todo:  failed to resolve path '/tmp/.tmpMnfzxX/.git/worktrees/worktree/': No such file or directory
                    let worktree = Worktree::open_from_repository(&repository)?;
                    Ok(Repository::open_from_worktree(&worktree).unwrap())
                } else {
                    Ok(repository)
                }
            }
            Err(_) => Err(SemVerError::RepositoryError {
                message: format!("Path {:?} is not a repository", path),
            }),
        }
    }
}

pub struct HistoryAnalyser {}

impl Analyser for HistoryAnalyser {
    fn run<P: AsRef<Path>, A: AnalyserPoint>(path: P, point: A) -> Result<Semantic, SemVerError> {
        let full_path = std::fs::canonicalize(path)?;
        log::debug!(
            "Calculate semantic version for repository at path: {:?}",
            full_path
        );
        let repository = Self::get_repository(full_path)?;
        let mut revwalk = repository.revwalk()?;

        // let mut tag: Option<Tag> = None;
        match point.since() {
            Some(commit) => {
                revwalk.push(commit)?;
                // tag = Some(repository.find_tag(commit)?);
            }
            None => revwalk.push_head()?,
        };
        let mut builder = Semantic::builder();

        if let Some(start) = &point.version_identifier() {
            builder.previous_version(start)?;
        }

        builder.is_prerelease(repository.head()?.shorthand().unwrap_or(""));

        if let Some(version) = point.version_identifier() {
            // if let Some(tag_name) = tag.name() {
                builder.previous_version(&version)?;
            // }
        }

        for commit_id in revwalk {
            let commit_id = commit_id?;
            let commit = repository.find_commit(commit_id)?;
            builder.analyze_commit(commit);

            if builder.has_major_release() {
                log::debug!("Commits contain major release. Stop search here.");
                break;
            }
        }
        // todo: set prerelease based on branch and configuration
        builder.calculate_version();
        Ok(builder.build())
    }
}

// impl History {
//     pub fn analyze<P: AsRef<Path>>(
//         path: P,
//         since: Option<Oid>,
//         start_version: Option<String>,
//     ) -> Result<Semantic, SemVerError> {
//         let full_path = std::fs::canonicalize(path)?;
//         log::debug!(
//             "Calculate semantic version for repository at path: {:?}",
//             full_path
//         );
//         let repository = History::get_repository(full_path)?;
//         let mut revwalk = repository.revwalk()?;

//         let mut tag: Option<Tag> = None;
//         match since {
//             Some(commit) => {
//                 revwalk.push(commit)?;
//                 tag = Some(repository.find_tag(commit)?);
//             }
//             None => revwalk.push_head()?,
//         };

//         let mut builder = Semantic::builder();

//         if let Some(start) = start_version {
//             builder.previous_version(&start)?;
//         }

//         builder.is_prerelease(repository.head()?.shorthand().unwrap_or(""));

//         if let Some(tag) = tag {
//             if let Some(tag_name) = tag.name() {
//                 builder.previous_version(tag_name)?;
//             }
//         }

//         for commit_id in revwalk {
//             let commit_id = commit_id?;
//             let commit = repository.find_commit(commit_id)?;
//             builder.analyze_commit(commit);

//             if builder.has_major_release() {
//                 log::debug!("Commits contain major release. Stop search here.");
//                 break;
//             }
//         }
//         // todo: set prerelease based on branch and configuration
//         builder.calculate_version();
//         Ok(builder.build())
//     }

//     pub fn get_repository(path: PathBuf) -> Result<Repository, SemVerError> {
//         match Repository::open(&path) {
//             Ok(repository) => {
//                 if repository.is_worktree() {
//                     log::info!(
//                         "Provided repository is a worktree. Try conversion finding repository."
//                     );
//                     // panic!("worktrees are not supported!") // Todo:  failed to resolve path '/tmp/.tmpMnfzxX/.git/worktrees/worktree/': No such file or directory
//                     let worktree = Worktree::open_from_repository(&repository)?;
//                     Ok(Repository::open_from_worktree(&worktree).unwrap())
//                 } else {
//                     Ok(repository)
//                 }
//             }
//             Err(_) => Err(SemVerError::RepositoryError {
//                 message: format!("Path {:?} is not a repository", path),
//             }),
//         }
//     }
// }

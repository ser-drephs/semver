use std::path::Path;

use git2::{Oid, Repository, Tag, Worktree};

use crate::{error::SemVerError, semantic::Semantic};

pub struct History {}

impl History {
    pub fn analyze<P: AsRef<Path>>(path: P, since: Option<Oid>) -> Result<Semantic, SemVerError> {
        log::debug!(
            "Calculate semantic version for repository at path: {:?}",
            path.as_ref()
        );
        let repository = History::get_repository_from_worktree(Repository::open(&path).unwrap())?;
        let mut revwalk = repository.revwalk()?;

        let mut tag: Option<Tag> = None;
        match since {
            Some(commit) => {
                revwalk.push(commit)?;
                tag = Some(repository.find_tag(commit)?);
            }
            None => revwalk.push_head()?,
        };

        let mut builder = Semantic::builder();

        builder.is_prerelease(repository.head()?.shorthand().unwrap_or(""));

        if let Some(tag) = tag {
            if let Some(tag_name) = tag.name(){
                builder.previous_version(tag_name)?;
            }
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

    fn get_repository_from_worktree(repository: Repository) -> Result<Repository, SemVerError> {
        if repository.is_worktree() {
            log::info!("Provided repository is a worktree. Try conversion finding repository.");
            // panic!("worktrees are not supported!") // Todo:  failed to resolve path '/tmp/.tmpMnfzxX/.git/worktrees/worktree/': No such file or directory
            let worktree = Worktree::open_from_repository(&repository)?;
            Ok(Repository::open_from_worktree(&worktree).unwrap())
        } else {
            Ok(repository)
        }
    }
}

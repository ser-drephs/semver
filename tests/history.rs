use std::{
    env,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use git2::{Commit, ObjectType, Oid, Repository, Signature};
use tempfile::{tempdir, TempDir};

use semver_calc::error::SemVerError;
use test_context::{test_context, TestContext};

fn find_last_commit(repo: &Repository) -> Result<Commit, git2::Error> {
    let obj = repo.head()?.resolve()?.peel(ObjectType::Commit)?;
    obj.into_commit()
        .map_err(|_| git2::Error::from_str("Couldn't find commit"))
}

fn add_and_commit(
    repo: &Repository,
    path: &Path,
    message: &str,
) -> core::result::Result<Oid, git2::Error> {
    let mut index = repo.index()?;
    index.add_path(path)?;
    let oid = index.write_tree()?;
    let signature = Signature::now("test", "test@test.ing")?;
    let tree = repo.find_tree(oid)?;

    match find_last_commit(repo) {
        Ok(res) => repo.commit(
            Some("HEAD"), //  point HEAD to our new commit
            &signature,   // author
            &signature,   // committer
            message,      // commit message
            &tree,        // tree
            &[&res],
        ),
        Err(_) => repo.commit(
            Some("HEAD"), //  point HEAD to our new commit
            &signature,   // author
            &signature,   // committer
            message,      // commit message
            &tree,        // tree
            &[],
        ),
    }
}

fn write_test_file(path: &Path) -> Result<(), SemVerError> {
    Write::write_all(
        &mut BufWriter::new(File::create(&path).unwrap()),
        "content".as_bytes(),
    )?;
    Ok(())
}

/// `path` should be relative to `repo`.
fn commit_test_file(
    repo: &Repository,
    path: &Path,
    message: &str,
) -> core::result::Result<Oid, git2::Error> {
    let root = repo.path().join("..");
    write_test_file(&root.join(path)).unwrap();
    add_and_commit(repo, path, message)
}

fn create_tag(
    repo: &Repository,
    name: &str,
    commit: &Commit,
) -> core::result::Result<Oid, git2::Error> {
    repo.tag_lightweight(name, commit.as_object(), false)
}

// fn commit_test_file_to_worktree(
//     worktree: &Worktree,
//     path: &Path,
//     message: &str,
// ) -> core::result::Result<Oid, git2::Error> {
//     let root = worktree.path();
//     write_test_file(&root.join(path)).unwrap();
//     add_and_commit(&Repository::open_from_worktree(worktree)?, path, message)
// }

/// Todo: add to support cargo
fn logger() {
    env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
}

#[cfg(test)]
mod given_path_is_repository {
    use semver_calc::history::{Analyser, CommitAnalyserPoint, HistoryAnalyser, TagAnalyserPoint};

    use super::*;

    struct RepositoryContext {
        dir: TempDir,
        repo: Repository,
    }

    impl TestContext for RepositoryContext {
        fn setup() -> RepositoryContext {
            let temp_dir = tempdir().unwrap();
            logger();
            // std::env::set_current_dir(&temp_dir.path()).unwrap();
            let repo = Repository::init(&temp_dir).unwrap();
            commit_test_file(&repo, &PathBuf::from("first.txt"), "chore: initial commit").unwrap();
            commit_test_file(&repo, &PathBuf::from("sample.rs"), "feat: impl feature").unwrap();
            RepositoryContext {
                dir: temp_dir,
                repo,
            }
        }

        fn teardown(self) {
            self.dir.close().unwrap();
        }
    }

    #[test_context(RepositoryContext)]
    #[test]
    fn when_feat_commit_exists_then_semantic_minor_is_set(ctx: &mut RepositoryContext) {
        let semantic = HistoryAnalyser::run(
            &ctx.dir,
            CommitAnalyserPoint {
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!semantic.major);
        assert!(semantic.minor);
        assert!(!semantic.patch);
    }

    #[test_context(RepositoryContext)]
    #[test]
    fn when_feat_and_fix_commit_exists_then_semantic_minor_is_set(ctx: &mut RepositoryContext) {
        commit_test_file(&ctx.repo, &PathBuf::from("sample-fix.rs"), "fix: feature").unwrap();
        let semantic = HistoryAnalyser::run(
            &ctx.dir,
            CommitAnalyserPoint {
                ..Default::default()
            },
        )
        .unwrap();
        assert!(!semantic.major);
        assert!(semantic.minor);
        assert!(semantic.patch);
    }

    #[test_context(RepositoryContext)]
    #[test]
    fn when_tag_exists_then_semantic_whatever(ctx: &mut RepositoryContext) {
        let commit_id =
            commit_test_file(&ctx.repo, &PathBuf::from("sample-fix.rs"), "fix: feature").unwrap();
        let commit = &ctx.repo.find_commit(commit_id).unwrap();
        create_tag(&ctx.repo, "v1.2.3", commit).unwrap();
        commit_test_file(
            &ctx.repo,
            &PathBuf::from("seconf.txt"),
            "chore: initial commit",
        )
        .unwrap();
        commit_test_file(
            &ctx.repo,
            &PathBuf::from("sample2.rs"),
            "feat: impl feature",
        )
        .unwrap();
        let point = TagAnalyserPoint::new(Some("v1.2.3"), &ctx.repo).unwrap();
        let semantic =
            HistoryAnalyser::run(&ctx.dir, point )
                .unwrap();
        assert!(!semantic.major);
        assert!(semantic.minor);
        assert!(semantic.patch);
        assert_eq!("1.3.0", semantic.version.to_string())
    }
}

// mod when_path_is_worktree{
//     use super::*;

//     struct WorktreeContext {
//         dir: TempDir,
//         worktree: PathBuf
//     }

//     impl TestContext for WorktreeContext {
//         fn setup() -> WorktreeContext {
//             let temp_dir = tempdir().unwrap();
//             // std::env::set_current_dir(&temp_dir.path()).unwrap();
//             let repo = Repository::init(&temp_dir).unwrap();
//             commit_test_file(&repo, &PathBuf::from("first.txt"), "chore: initial commit").unwrap();
//             commit_test_file(&repo, &PathBuf::from("sample.rs"), "feat: impl feature").unwrap();

//             let temp_dir = tempdir().unwrap();
//             let worktree_dir = temp_dir.path().join("worktree");
//             let worktree = repo.worktree("worktree", &worktree_dir, None).unwrap();
//             commit_test_file_to_worktree(&worktree, &PathBuf::from("sample_worktree.rs"), "fix: feature").unwrap();

//             WorktreeContext { dir: temp_dir, worktree: worktree_dir }
//         }

//         fn teardown(self) {
//             self.dir.close().unwrap();
//         }
//     }

//     #[test_context(WorktreeContext)]
//     #[test]
//     fn then_history_is_gathered_and_items_should_appear(ctx: &mut RepositoryContext) {
//         let commits = History::read_all(&ctx.worktree).unwrap();
//         assert_eq!(3, commits.len());
//     }
// }

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemVerError {
    #[error("dafuq")]
    Unknown,
    #[error("git library error")]
    GitError(#[from] git2::Error),
    #[error("general failure")]
    Error(#[from] std::io::Error)
}

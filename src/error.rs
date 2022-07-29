use thiserror::Error;

#[derive(Error, Debug)]
pub enum SemVerError {
    #[error("dafuq")]
    Unknown,
    #[error("git library error")]
    GitError(#[from] git2::Error),
    #[error("general failure")]
    Error(#[from] std::io::Error),
    #[error("semantic version error")]
    SemVerError(#[from] semver::Error),
    #[error("semantic version error: {message:?}")]
    SemanticError { message: String },
    #[error("repository error: {message:?}")]
    RepositoryError { message: String },
    #[error("logger error")]
    LoggerError(#[from] log::SetLoggerError),
}

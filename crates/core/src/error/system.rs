use thiserror::Error;

#[derive(Debug, Error)]
pub enum SystemError {
    #[error("mutex poisoned: {0}")]
    MutexPoisoned(String),
}

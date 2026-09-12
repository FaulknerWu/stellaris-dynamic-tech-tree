pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("rule engine error: {0}")]
    RuleEngine(String),
    #[error("domain invariant error: {0}")]
    Invariant(String),
}

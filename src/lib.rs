pub mod balance;
pub mod transaction;

// Use thiserror instead of anyhow since the prompt seemed
// to suggest that this engine should be easy to integrate
// with a server. Otherwise, for a plain CLI tool I would
// probably just use anyhow.
#[derive(thiserror::Error, Debug)]
pub enum PaymentEngineError {
    #[error("usage error")]
    Usage,
    #[error("{0}")]
    Csv(String),
    #[error("unknown error")]
    Unknown,
}

pub type PaymentEngineResult<T> = std::result::Result<T, PaymentEngineError>;

pub mod prelude {
    pub use crate::{
        PaymentEngineError, PaymentEngineResult,
        balance::Balance,
        transaction::{Transaction, TransactionType},
    };
}

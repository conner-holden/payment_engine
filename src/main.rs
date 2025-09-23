use std::{env, path::PathBuf};

use tracing::error;

mod transaction;

// Use thiserror instead of anyhow since the prompt seemed
// to suggest that this engine should be easy to integrate
// with a server. Otherwise, for a plain CLI tool I would
// probably just use anyhow.
#[derive(thiserror::Error, Debug)]
pub enum PaymentEngineError {
    #[error("unknown error")]
    Unknown,
}

pub type Result<T> = std::result::Result<T, PaymentEngineError>;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .init();
    // Since the requirement is for a single argument,
    // no need for clap!
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        error!("Usage: {} <path>", args[0]);
        return Err(PaymentEngineError::Unknown);
    }
    Ok(())
}

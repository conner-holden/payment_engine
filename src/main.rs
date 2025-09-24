use std::{env, path::PathBuf};

use payment_engine::{PaymentEngineError, PaymentEngineResult};
use polars::prelude::*;
use tracing::error;

fn main() -> PaymentEngineResult<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .init();

    // Since the requirement is for a single argument,
    // no need for clap!
    let args: Vec<String> = env::args().collect();
    // Gracefully handle any extra arguments and flags
    let Some(path) = args.get(1).map(Into::<PathBuf>::into) else {
        error!("Usage: {} <path>", args[0]);
        return Err(PaymentEngineError::Unknown);
    };

    let df = CsvReadOptions::default()
        .try_into_reader_with_file_path(Some(path))
        .and_then(|reader| reader.finish())
        // Not as clean as .map_err(PaymentEngineError::Csv), but
        // the outputed error is much nicer.
        .map_err(|err| PaymentEngineError::Csv(err.to_string()))?;

    Ok(())
}

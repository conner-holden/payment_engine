use std::{collections::HashMap, env, path::PathBuf};

use payment_engine::prelude::*;
use rust_decimal::Decimal;
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
        return Err(PaymentEngineError::Usage);
    };

    let mut accounts: HashMap<u16, Balance> = HashMap::new();
    let mut transactions: HashMap<u32, Transaction> = HashMap::new();

    let mut rdr =
        csv::Reader::from_path(path).map_err(|err| PaymentEngineError::Csv(err.to_string()))?;

    use TransactionType::*;
    for result in rdr.deserialize() {
        let Ok(transaction): Result<Transaction, _> = result else {
            continue;
        };
        accounts
            .entry(transaction.client)
            .and_modify(|balance| {
                if balance.is_locked {
                    return;
                }
                let amount = match transaction.ty {
                    Deposit | Withdrawal => {
                        transactions.insert(transaction.id, transaction);
                        transaction.amount.unwrap_or(Decimal::ZERO)
                    }
                    _ => transactions
                        .get(&transaction.id)
                        .and_then(|t| t.amount)
                        .unwrap_or(Decimal::ZERO),
                };
                balance.commit(transaction.ty, amount);
            })
            .or_default();
    }

    dbg!(accounts);

    Ok(())
}

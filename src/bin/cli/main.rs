use std::{collections::HashMap, env, io, path::PathBuf};

use payment_engine::prelude::*;
use rust_decimal::Decimal;

#[derive(thiserror::Error)]
enum Error {
    #[error("expected `{0} <path>`")]
    Usage(String),
    #[error("{0}")]
    Csv(String),
}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

fn main() -> Result<(), Error> {
    // Since the requirement is for a single argument,
    // no need for clap!
    let args: Vec<String> = env::args().collect();
    // Gracefully handle any extra arguments and flags
    let Some(path) = args.get(1).map(Into::<PathBuf>::into) else {
        return Err(Error::Usage(args[0].clone()));
    };

    let mut accounts: HashMap<u16, Balance> = HashMap::new();
    let mut transactions: HashMap<u32, Transaction> = HashMap::new();

    let mut rdr = csv::Reader::from_path(path).map_err(|err| Error::Csv(err.to_string()))?;

    use TransactionType::*;
    for result in rdr.deserialize() {
        let Ok(transaction): Result<Transaction, _> = result else {
            continue;
        };
        accounts
            .entry(transaction.client)
            .and_modify(|balance| {
                // Skip locked balances
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
                // Apply transaction to balance
                balance.commit(transaction.ty, amount);
                // Lock balance on charge back
                if matches!(transaction.ty, Chargeback) {
                    balance.is_locked = true;
                }
            })
            .or_insert(Balance::new(transaction.client));
    }

    let mut wtr = csv::Writer::from_writer(io::stdout());
    for balance in accounts.values() {
        let _ = wtr.serialize(balance);
    }
    wtr.flush().unwrap();

    Ok(())
}

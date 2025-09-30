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

// Forward debug to display for cleaner CLI errors.
// AI: asked ChatGPT how to get CLI errors to use thiserror's
// display implementation.
impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

// AI: asked ChatGPT how to have multiple binaries in a single crate where one is
// the default (so automated tests don't break).
fn main() -> Result<(), Error> {
    // Since the requirement is for a single argument,
    // no need for clap!
    let args: Vec<String> = env::args().collect();
    // Gracefully handle any extra arguments and flags
    // AI: asked ChatGPT how to handle args as PathBuf without clap
    let Some(path) = args.get(1).map(Into::<PathBuf>::into) else {
        return Err(Error::Usage(args[0].clone()));
    };

    // These will grow without bound. Could be a memory issue for
    // large inputs, in which case it might be better to use disk-backed
    // HashMaps instead. That might break the tests though.
    // AI: asked Claude recommendation for handling arbitrarily large inputs.
    let mut accounts: HashMap<u16, Account> = HashMap::new();
    let mut transactions: HashMap<u32, Transaction> = HashMap::new();

    let mut rdr = csv::Reader::from_path(path).map_err(|err| Error::Csv(err.to_string()))?;

    use TransactionType::*;
    for result in rdr.deserialize() {
        let Ok(transaction): Result<Transaction, _> = result else {
            continue;
        };
        accounts
            .entry(transaction.client)
            .and_modify(|account| {
                // Skip locked accounts
                if account.is_locked {
                    return;
                }
                let amount = match transaction.ty {
                    Deposit | Withdrawal => {
                        transactions.insert(transaction.id, transaction);
                        transaction.amount.unwrap_or(Decimal::ZERO)
                    }
                    // Better to be explicit in case new transaction types are added
                    Dispute | Resolve | Chargeback => transactions
                        .get(&transaction.id)
                        .and_then(|t| t.amount)
                        .unwrap_or(Decimal::ZERO),
                };
                // Apply transaction to account
                account.commit(transaction.ty, amount);
                // Lock account on charge back
                if matches!(transaction.ty, Chargeback) {
                    account.is_locked = true;
                }
            })
            .or_insert(Account::new(transaction.client));
    }

    let mut wtr = csv::Writer::from_writer(io::stdout());
    for account in accounts.values() {
        let _ = wtr.serialize(account);
    }
    wtr.flush().unwrap();

    Ok(())
}

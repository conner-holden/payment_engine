use std::{collections::HashMap, io};

use payment_engine::prelude::*;
use rand::{
    Rng as _,
    seq::{IndexedRandom, IteratorRandom},
};
use rust_decimal::Decimal;
use strum::IntoEnumIterator as _;

const DECIMALS: u32 = 4;
const MAX_CLIENT_ID: u16 = 300;
const MIN_TX: usize = 5_000;
const MAX_TX: usize = 10_000;
const MIN_AMOUNT: i64 = -10_000 * 10_i64.pow(DECIMALS); // -$10,000
const MAX_AMOUNT: i64 = 10_000 * 10_i64.pow(DECIMALS); // $10,000

fn main() {
    let mut rng = rand::rng();
    let mut simple_txs: HashMap<u32, Transaction> = HashMap::new();
    let mut open_dispute_txs: HashMap<u32, Transaction> = HashMap::new();
    let tx_types: Vec<TransactionType> = TransactionType::iter().collect();

    let mut wtr = csv::Writer::from_writer(io::stdout());

    let total_tx = rng.random_range(MIN_TX..=MAX_TX);
    for i in 1..total_tx {
        let tx_type = tx_types.choose(&mut rng).unwrap();
        let tx = match *tx_type {
            TransactionType::Withdrawal => {
                let amount = Decimal::new(
                    rng.random_range(MIN_AMOUNT..=(-1 * 10_i64.pow(DECIMALS))),
                    DECIMALS,
                );
                let tx = Transaction {
                    amount: Some(amount),
                    // Transaction IDs are not necessarily ordered, so let's
                    // use the index for simplicity
                    id: i as u32,
                    client: rng.random_range(1..=MAX_CLIENT_ID),
                    ty: *tx_type,
                };
                simple_txs.insert(tx.id, tx);
                tx
            }
            TransactionType::Deposit => {
                let amount = Decimal::new(
                    rng.random_range(10_i64.pow(DECIMALS)..=MAX_AMOUNT),
                    DECIMALS,
                );
                let tx = Transaction {
                    amount: Some(amount),
                    id: i as u32,
                    client: rng.random_range(1..=MAX_CLIENT_ID),
                    ty: *tx_type,
                };
                simple_txs.insert(tx.id, tx);
                tx
            }
            TransactionType::Dispute => {
                let Some(source_tx) = simple_txs.values().choose(&mut rng) else {
                    continue;
                };
                let tx = Transaction {
                    amount: None,
                    ty: *tx_type,
                    ..*source_tx
                };
                open_dispute_txs.insert(tx.id, tx);
                tx
            }
            TransactionType::Resolve => {
                let Some(dispute_tx) = open_dispute_txs.values().choose(&mut rng).cloned() else {
                    continue;
                };
                let tx = Transaction {
                    ty: *tx_type,
                    ..dispute_tx
                };
                open_dispute_txs.remove(&dispute_tx.id);
                tx
            }
            TransactionType::Chargeback => {
                let Some(dispute_tx) = open_dispute_txs.values().choose(&mut rng).cloned() else {
                    continue;
                };
                let tx = Transaction {
                    ty: *tx_type,
                    ..dispute_tx
                };
                open_dispute_txs.remove(&dispute_tx.id);
                tx
            }
        };
        wtr.serialize(tx).unwrap();
    }
    wtr.flush().unwrap();
}

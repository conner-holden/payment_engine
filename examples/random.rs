use std::collections::HashMap;

use payment_engine::prelude::*;
use rand::{
    Rng as _,
    seq::{IndexedRandom, IteratorRandom},
};
use rust_decimal::Decimal;
use strum::IntoEnumIterator as _;

const DECIMALS: u32 = 4;
const MIN_CLIENTS: u16 = 1;
const MAX_CLIENTS: u16 = 10;
const MIN_TX: usize = 100;
const MAX_TX: usize = 1000;
const MIN_AMOUNT: i64 = -10_000 * 10_i64.pow(DECIMALS); // -$10,000
const MAX_AMOUNT: i64 = 10_000 * 10_i64.pow(DECIMALS); // $10,000

fn main() {
    let mut rng = rand::rng();
    let mut simple_txs: HashMap<u32, Transaction> = HashMap::new();
    let mut all_txs: Vec<Transaction> = Vec::new();
    let mut open_dispute_txs: HashMap<u32, Transaction> = HashMap::new();
    let tx_types: Vec<TransactionType> = TransactionType::iter().collect();

    let mut balances: HashMap<u16, Balance> = HashMap::new();

    for i in MIN_TX..rng.random_range(MIN_TX..=MAX_TX) {
        let tx_type = tx_types.choose(&mut rng).unwrap();
        match *tx_type {
            TransactionType::Withdrawal => {
                let amount = Decimal::new(rng.random_range(MIN_AMOUNT..=0), DECIMALS);
                let tx = Transaction {
                    amount: Some(amount),
                    // Transaction IDs are not necessarily ordered, so let's
                    // use the index for simplicity
                    id: i as u32,
                    client: rng.random_range(MIN_CLIENTS..=MAX_CLIENTS),
                    ty: *tx_type,
                };
                simple_txs.insert(tx.id, tx);
                all_txs.push(tx);
                balances
                    .entry(tx.client)
                    .and_modify(|b| b.withdraw(amount))
                    .or_default();
            }
            TransactionType::Deposit => {
                let amount = Decimal::new(rng.random_range(0..=MAX_AMOUNT), DECIMALS);
                let tx = Transaction {
                    amount: Some(amount),
                    id: i as u32,
                    client: rng.random_range(MIN_CLIENTS..=MAX_CLIENTS),
                    ty: *tx_type,
                };
                simple_txs.insert(tx.id, tx);
                all_txs.push(tx);
                balances
                    .entry(tx.client)
                    .and_modify(|b| b.deposit(amount))
                    .or_default();
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
                all_txs.push(tx);
                balances
                    .entry(tx.client)
                    .and_modify(|b| b.dispute(source_tx.amount.unwrap()))
                    .or_default();
            }
            TransactionType::Resolution => {
                let Some(dispute_tx) = open_dispute_txs.values().choose(&mut rng).cloned() else {
                    continue;
                };
                let Some(source_tx) = simple_txs.get(&dispute_tx.id) else {
                    continue;
                };
                let tx = Transaction {
                    ty: *tx_type,
                    ..dispute_tx
                };
                all_txs.push(tx);
                open_dispute_txs.remove(&dispute_tx.id);
                balances
                    .entry(tx.client)
                    .and_modify(|b| b.resolve(source_tx.amount.unwrap()))
                    .or_default();
            }
            TransactionType::Chargeback => {
                let Some(dispute_tx) = open_dispute_txs.values().choose(&mut rng).cloned() else {
                    continue;
                };
                let Some(source_tx) = simple_txs.get(&dispute_tx.id) else {
                    continue;
                };
                let tx = Transaction {
                    ty: *tx_type,
                    ..dispute_tx
                };
                all_txs.push(tx);
                open_dispute_txs.remove(&dispute_tx.id);
                balances
                    .entry(tx.client)
                    .and_modify(|b| b.resolve(source_tx.amount.unwrap()))
                    .or_default();
            }
        };
    }
    dbg!(balances);
}

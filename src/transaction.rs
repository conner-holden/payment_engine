use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

// Struct fields are ordered to minimize padding
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    // Although f32 works for floats with precision up to four decimal numbers,
    // Decimal is more accurate and well-suited to financial calculations.
    // So while I think Decimal is overkill for a toy payment engine,
    // the 4x size would be well worth it for a production payment engine that
    // demands 100% accuracy.
    pub amount: Option<Decimal>,
    #[serde(rename = "tx")]
    pub id: u32,
    pub client: u16,
    #[serde(rename = "type")]
    pub ty: TransactionType,
}

#[derive(Clone, Copy, Debug, EnumIter, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    /// Credit to the client's asset account. Increases available and total funds.
    Deposit,
    /// Debit to the client's asset account. Decreases available and total funds.
    Withdrawal,
    /// Client's claim to an erroneous charge. Decreases available, increases held,
    /// and maintains total funds.
    Dispute,
    /// Finalizes a dispute by committing disputed transaction. Decreases held,
    /// increases available, and maintains total funds.
    Resolve,
    /// Finalizes a dispute by reversing disputed transaction. Decreases held and
    /// total funds. Should lock an account.
    Chargeback,
}

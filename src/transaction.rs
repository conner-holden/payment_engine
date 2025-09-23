use rust_decimal::Decimal;

// Struct fields are ordered to minimize padding
pub struct Transaction {
    // Although f32 works for floats with precision up to four decimal numbers,
    // Decimal is more accurate and well-suited to financial calculations.
    // So while I think Decimal is overkill for a toy payment engine,
    // the 4x size would be well worth it for a production payment engine that
    // demands 100% accuracy.
    //
    // The amount is optional to account for transaction types other than deposits
    // and withdrawls.
    pub amount: Option<Decimal>,
    pub tx: u32,
    pub client: u16,
    pub ty: TranscationType,
}

pub enum TranscationType {
    /// Credit to the client's asset account. Increases available and total funds.
    Deposit,
    /// Debit to the client's asset account. Decreases available and total funds.
    Withdrawl,
    /// Client's claim to an erroneous charge. Decreases available, increases held,
    /// and maintains total funds.
    Dispute,
    /// Resolves a dispute by committing disputed transaction. Decreases held,
    /// increases available, and maintains total funds.
    Resolution,
    /// Resolves a dispute by reversing disputed transaction. Decreases held and
    /// total funds.
    Chargeback,
}

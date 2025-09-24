use crate::transaction::TransactionType;

use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Balance {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    #[serde(rename = "locked")]
    pub is_locked: bool,
}

impl Balance {
    pub fn new(client: u16) -> Self {
        Self {
            client,
            ..Default::default()
        }
    }

    #[inline]
    pub fn commit(&mut self, transaction_type: TransactionType, amount: Decimal) {
        if self.is_locked {
            return;
        }
        use TransactionType::*;
        match transaction_type {
            Withdrawal => self.withdraw(amount),
            Deposit => self.deposit(amount),
            Dispute => self.dispute(amount),
            Resolve => self.resolve(amount),
            Chargeback => self.chargeback(amount),
        }
        debug_assert_eq!(self.total - self.held, self.available);
    }

    #[inline]
    fn withdraw(&mut self, amount: Decimal) {
        if amount > self.available {
            return;
        }
        self.available -= amount;
        self.total -= amount;
    }

    #[inline]
    fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
        self.total += amount;
    }

    #[inline]
    fn dispute(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
    }

    #[inline]
    fn resolve(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    #[inline]
    fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
        self.total -= amount;
    }
}

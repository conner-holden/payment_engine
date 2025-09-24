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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_new_balance() {
        let balance = Balance::new(123);
        assert_eq!(balance.client, 123);
        assert_eq!(balance.available, Decimal::ZERO);
        assert_eq!(balance.held, Decimal::ZERO);
        assert_eq!(balance.total, Decimal::ZERO);
        assert!(!balance.is_locked);
    }

    #[test]
    fn test_deposit() {
        let mut balance = Balance::new(1);
        balance.commit(TransactionType::Deposit, Decimal::new(1005000, 4)); // 100.5000

        assert_eq!(balance.available, Decimal::new(1005000, 4));
        assert_eq!(balance.held, Decimal::ZERO);
        assert_eq!(balance.total, Decimal::new(1005000, 4));
    }

    #[test]
    fn test_withdrawal_sufficient_funds() {
        let mut balance = Balance::new(1);
        balance.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        balance.commit(TransactionType::Withdrawal, Decimal::new(500000, 4)); // 50.0000

        assert_eq!(balance.available, Decimal::new(500000, 4));
        assert_eq!(balance.held, Decimal::ZERO);
        assert_eq!(balance.total, Decimal::new(500000, 4));
    }

    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut balance = Balance::new(1);
        balance.commit(TransactionType::Deposit, Decimal::new(500000, 4)); // 50.0000
        balance.commit(TransactionType::Withdrawal, Decimal::new(1000000, 4)); // 100.0000

        // Should remain unchanged due to insufficient funds
        assert_eq!(balance.available, Decimal::new(500000, 4));
        assert_eq!(balance.held, Decimal::ZERO);
        assert_eq!(balance.total, Decimal::new(500000, 4));
    }

    #[test]
    fn test_dispute() {
        let mut balance = Balance::new(1);
        balance.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        balance.commit(TransactionType::Dispute, Decimal::new(300000, 4)); // 30.0000

        assert_eq!(balance.available, Decimal::new(700000, 4)); // 70.0000
        assert_eq!(balance.held, Decimal::new(300000, 4)); // 30.0000
        assert_eq!(balance.total, Decimal::new(1000000, 4)); // 100.0000
    }

    #[test]
    fn test_resolve() {
        let mut balance = Balance::new(1);
        balance.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        balance.commit(TransactionType::Dispute, Decimal::new(300000, 4)); // 30.0000
        balance.commit(TransactionType::Resolve, Decimal::new(300000, 4)); // 30.0000

        assert_eq!(balance.available, Decimal::new(1000000, 4)); // 100.0000
        assert_eq!(balance.held, Decimal::ZERO);
        assert_eq!(balance.total, Decimal::new(1000000, 4)); // 100.0000
    }

    #[test]
    fn test_chargeback() {
        let mut balance = Balance::new(1);
        balance.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        balance.commit(TransactionType::Dispute, Decimal::new(300000, 4)); // 30.0000
        balance.commit(TransactionType::Chargeback, Decimal::new(300000, 4)); // 30.0000

        assert_eq!(balance.available, Decimal::new(700000, 4)); // 70.0000
        assert_eq!(balance.held, Decimal::ZERO);
        assert_eq!(balance.total, Decimal::new(700000, 4)); // 70.0000
    }

    #[test]
    fn test_locked_account_ignores_transactions() {
        let mut balance = Balance::new(1);
        balance.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        balance.commit(TransactionType::Dispute, Decimal::new(300000, 4)); // 30.0000
        balance.commit(TransactionType::Chargeback, Decimal::new(300000, 4)); // 30.0000

        // After chargeback, account should be locked (this would typically be set externally)
        balance.is_locked = true;

        let available_before = balance.available;
        let held_before = balance.held;
        let total_before = balance.total;

        // These should all be ignored due to locked account
        balance.commit(TransactionType::Deposit, Decimal::new(500000, 4));
        balance.commit(TransactionType::Withdrawal, Decimal::new(100000, 4));
        balance.commit(TransactionType::Dispute, Decimal::new(100000, 4));
        balance.commit(TransactionType::Resolve, Decimal::new(100000, 4));

        assert_eq!(balance.available, available_before);
        assert_eq!(balance.held, held_before);
        assert_eq!(balance.total, total_before);
    }

    #[test]
    fn test_multiple_transactions() {
        let mut balance = Balance::new(1);

        // Multiple deposits
        balance.commit(TransactionType::Deposit, Decimal::new(500000, 4)); // 50.0000
        balance.commit(TransactionType::Deposit, Decimal::new(250000, 4)); // 25.0000
        balance.commit(TransactionType::Deposit, Decimal::new(250000, 4)); // 25.0000

        assert_eq!(balance.total, Decimal::new(1000000, 4)); // 100.0000
        assert_eq!(balance.available, Decimal::new(1000000, 4)); // 100.0000

        // Withdrawal
        balance.commit(TransactionType::Withdrawal, Decimal::new(150000, 4)); // 15.0000

        assert_eq!(balance.total, Decimal::new(850000, 4)); // 85.0000
        assert_eq!(balance.available, Decimal::new(850000, 4)); // 85.0000

        // Dispute
        balance.commit(TransactionType::Dispute, Decimal::new(200000, 4)); // 20.0000

        assert_eq!(balance.total, Decimal::new(850000, 4)); // 85.0000
        assert_eq!(balance.available, Decimal::new(650000, 4)); // 65.0000
        assert_eq!(balance.held, Decimal::new(200000, 4)); // 20.0000
    }

    #[test]
    fn test_balance_invariants() {
        let mut balance = Balance::new(1);

        // Test that total == available + held always holds
        balance.commit(TransactionType::Deposit, Decimal::new(1000000, 4));
        assert_eq!(balance.total, balance.available + balance.held);

        balance.commit(TransactionType::Dispute, Decimal::new(300000, 4));
        assert_eq!(balance.total, balance.available + balance.held);

        balance.commit(TransactionType::Resolve, Decimal::new(300000, 4));
        assert_eq!(balance.total, balance.available + balance.held);

        balance.commit(TransactionType::Dispute, Decimal::new(200000, 4));
        assert_eq!(balance.total, balance.available + balance.held);

        balance.commit(TransactionType::Chargeback, Decimal::new(200000, 4));
        assert_eq!(balance.total, balance.available + balance.held);
    }

    #[test]
    fn test_zero_amount_transactions() {
        let mut balance = Balance::new(1);
        balance.commit(TransactionType::Deposit, Decimal::new(500000, 4)); // 50.0000

        let original_available = balance.available;
        let original_held = balance.held;
        let original_total = balance.total;

        // Zero amount transactions should not change anything
        balance.commit(TransactionType::Deposit, Decimal::ZERO);
        balance.commit(TransactionType::Withdrawal, Decimal::ZERO);
        balance.commit(TransactionType::Dispute, Decimal::ZERO);
        balance.commit(TransactionType::Resolve, Decimal::ZERO);
        balance.commit(TransactionType::Chargeback, Decimal::ZERO);

        assert_eq!(balance.available, original_available);
        assert_eq!(balance.held, original_held);
        assert_eq!(balance.total, original_total);
    }

    #[test]
    fn test_precision_handling() {
        let mut balance = Balance::new(1);

        // Test with 4 decimal places
        balance.commit(TransactionType::Deposit, Decimal::new(123456, 4)); // 12.3456

        assert_eq!(balance.available, Decimal::new(123456, 4));
        assert_eq!(balance.total, Decimal::new(123456, 4));

        balance.commit(TransactionType::Withdrawal, Decimal::new(3456, 4)); // 0.3456

        assert_eq!(balance.available, Decimal::new(120000, 4)); // 12.0000
        assert_eq!(balance.total, Decimal::new(120000, 4));
    }
}

use crate::transaction::TransactionType;

use rust_decimal::Decimal;
use serde::Serialize;

#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct Account {
    pub client: u16,
    pub available: Decimal,
    pub held: Decimal,
    pub total: Decimal,
    #[serde(rename = "locked")]
    pub is_locked: bool,
}

impl Account {
    /// Create a new account for a `client`.
    pub fn new(client: u16) -> Self {
        Self {
            client,
            ..Default::default()
        }
    }

    /// Commit a transaction to the account.
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

    /// Debit to the client's asset account. Decreases available and total funds.
    #[inline]
    fn withdraw(&mut self, amount: Decimal) {
        if amount > self.available {
            return;
        }
        self.available -= amount;
        self.total -= amount;
    }

    /// Credit to the client's asset account. Increases available and total funds.
    #[inline]
    fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
        self.total += amount;
    }

    /// Client's claim to an erroneous charge. Decreases available, increases held,
    /// and maintains total funds.
    #[inline]
    fn dispute(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
    }

    /// Finalizes a dispute by committing disputed transaction. Decreases held,
    /// increases available, and maintains total funds.
    #[inline]
    fn resolve(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
    }

    /// Finalizes a dispute by reversing disputed transaction. Decreases held and
    /// total funds. Should lock an account.
    #[inline]
    fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
        self.total -= amount;
    }
}

// AI: asked Claude to generate all test cases. RIP TDD.
#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn test_new_account() {
        let account = Account::new(123);
        assert_eq!(account.client, 123);
        assert_eq!(account.available, Decimal::ZERO);
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::ZERO);
        assert!(!account.is_locked);
    }

    #[test]
    fn test_deposit() {
        let mut account = Account::new(1);
        account.commit(TransactionType::Deposit, Decimal::new(1005000, 4)); // 100.5000

        assert_eq!(account.available, Decimal::new(1005000, 4));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(1005000, 4));
    }

    #[test]
    fn test_withdrawal_sufficient_funds() {
        let mut account = Account::new(1);
        account.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        account.commit(TransactionType::Withdrawal, Decimal::new(500000, 4)); // 50.0000

        assert_eq!(account.available, Decimal::new(500000, 4));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(500000, 4));
    }

    #[test]
    fn test_withdrawal_insufficient_funds() {
        let mut account = Account::new(1);
        account.commit(TransactionType::Deposit, Decimal::new(500000, 4)); // 50.0000
        account.commit(TransactionType::Withdrawal, Decimal::new(1000000, 4)); // 100.0000

        // Should remain unchanged due to insufficient funds
        assert_eq!(account.available, Decimal::new(500000, 4));
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(500000, 4));
    }

    #[test]
    fn test_dispute() {
        let mut account = Account::new(1);
        account.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        account.commit(TransactionType::Dispute, Decimal::new(300000, 4)); // 30.0000

        assert_eq!(account.available, Decimal::new(700000, 4)); // 70.0000
        assert_eq!(account.held, Decimal::new(300000, 4)); // 30.0000
        assert_eq!(account.total, Decimal::new(1000000, 4)); // 100.0000
    }

    #[test]
    fn test_resolve() {
        let mut account = Account::new(1);
        account.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        account.commit(TransactionType::Dispute, Decimal::new(300000, 4)); // 30.0000
        account.commit(TransactionType::Resolve, Decimal::new(300000, 4)); // 30.0000

        assert_eq!(account.available, Decimal::new(1000000, 4)); // 100.0000
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(1000000, 4)); // 100.0000
    }

    #[test]
    fn test_chargeback() {
        let mut account = Account::new(1);
        account.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        account.commit(TransactionType::Dispute, Decimal::new(300000, 4)); // 30.0000
        account.commit(TransactionType::Chargeback, Decimal::new(300000, 4)); // 30.0000

        assert_eq!(account.available, Decimal::new(700000, 4)); // 70.0000
        assert_eq!(account.held, Decimal::ZERO);
        assert_eq!(account.total, Decimal::new(700000, 4)); // 70.0000
    }

    #[test]
    fn test_locked_account_ignores_transactions() {
        let mut account = Account::new(1);
        account.commit(TransactionType::Deposit, Decimal::new(1000000, 4)); // 100.0000
        account.commit(TransactionType::Dispute, Decimal::new(300000, 4)); // 30.0000
        account.commit(TransactionType::Chargeback, Decimal::new(300000, 4)); // 30.0000

        // After chargeback, account should be locked (this would typically be set externally)
        account.is_locked = true;

        let available_before = account.available;
        let held_before = account.held;
        let total_before = account.total;

        // These should all be ignored due to locked account
        account.commit(TransactionType::Deposit, Decimal::new(500000, 4));
        account.commit(TransactionType::Withdrawal, Decimal::new(100000, 4));
        account.commit(TransactionType::Dispute, Decimal::new(100000, 4));
        account.commit(TransactionType::Resolve, Decimal::new(100000, 4));

        assert_eq!(account.available, available_before);
        assert_eq!(account.held, held_before);
        assert_eq!(account.total, total_before);
    }

    #[test]
    fn test_multiple_transactions() {
        let mut account = Account::new(1);

        // Multiple deposits
        account.commit(TransactionType::Deposit, Decimal::new(500000, 4)); // 50.0000
        account.commit(TransactionType::Deposit, Decimal::new(250000, 4)); // 25.0000
        account.commit(TransactionType::Deposit, Decimal::new(250000, 4)); // 25.0000

        assert_eq!(account.total, Decimal::new(1000000, 4)); // 100.0000
        assert_eq!(account.available, Decimal::new(1000000, 4)); // 100.0000

        // Withdrawal
        account.commit(TransactionType::Withdrawal, Decimal::new(150000, 4)); // 15.0000

        assert_eq!(account.total, Decimal::new(850000, 4)); // 85.0000
        assert_eq!(account.available, Decimal::new(850000, 4)); // 85.0000

        // Dispute
        account.commit(TransactionType::Dispute, Decimal::new(200000, 4)); // 20.0000

        assert_eq!(account.total, Decimal::new(850000, 4)); // 85.0000
        assert_eq!(account.available, Decimal::new(650000, 4)); // 65.0000
        assert_eq!(account.held, Decimal::new(200000, 4)); // 20.0000
    }

    #[test]
    fn test_account_invariants() {
        let mut account = Account::new(1);

        // Test that total == available + held always holds
        account.commit(TransactionType::Deposit, Decimal::new(1000000, 4));
        assert_eq!(account.total, account.available + account.held);

        account.commit(TransactionType::Dispute, Decimal::new(300000, 4));
        assert_eq!(account.total, account.available + account.held);

        account.commit(TransactionType::Resolve, Decimal::new(300000, 4));
        assert_eq!(account.total, account.available + account.held);

        account.commit(TransactionType::Dispute, Decimal::new(200000, 4));
        assert_eq!(account.total, account.available + account.held);

        account.commit(TransactionType::Chargeback, Decimal::new(200000, 4));
        assert_eq!(account.total, account.available + account.held);
    }

    #[test]
    fn test_zero_amount_transactions() {
        let mut account = Account::new(1);
        account.commit(TransactionType::Deposit, Decimal::new(500000, 4)); // 50.0000

        let original_available = account.available;
        let original_held = account.held;
        let original_total = account.total;

        // Zero amount transactions should not change anything
        account.commit(TransactionType::Deposit, Decimal::ZERO);
        account.commit(TransactionType::Withdrawal, Decimal::ZERO);
        account.commit(TransactionType::Dispute, Decimal::ZERO);
        account.commit(TransactionType::Resolve, Decimal::ZERO);
        account.commit(TransactionType::Chargeback, Decimal::ZERO);

        assert_eq!(account.available, original_available);
        assert_eq!(account.held, original_held);
        assert_eq!(account.total, original_total);
    }

    #[test]
    fn test_precision_handling() {
        let mut account = Account::new(1);

        // Test with 4 decimal places
        account.commit(TransactionType::Deposit, Decimal::new(123456, 4)); // 12.3456

        assert_eq!(account.available, Decimal::new(123456, 4));
        assert_eq!(account.total, Decimal::new(123456, 4));

        account.commit(TransactionType::Withdrawal, Decimal::new(3456, 4)); // 0.3456

        assert_eq!(account.available, Decimal::new(120000, 4)); // 12.0000
        assert_eq!(account.total, Decimal::new(120000, 4));
    }
}

use rust_decimal::Decimal;

#[derive(Clone, Copy, Debug, Default)]
pub struct Balance {
    pub available: Decimal,
    pub total: Decimal,
    pub held: Decimal,
}

impl Balance {
    #[inline]
    pub fn withdraw(&mut self, amount: Decimal) {
        if amount > self.available {
            return;
        }
        self.available -= amount;
        self.total -= amount;
        debug_assert_eq!(self.total - self.held, self.available);
    }

    #[inline]
    pub fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
        self.total += amount;
        debug_assert_eq!(self.total - self.held, self.available);
    }

    #[inline]
    pub fn dispute(&mut self, amount: Decimal) {
        self.available -= amount;
        self.held += amount;
        debug_assert_eq!(self.total - self.held, self.available);
    }

    #[inline]
    pub fn resolve(&mut self, amount: Decimal) {
        self.held -= amount;
        self.available += amount;
        debug_assert_eq!(self.total - self.held, self.available);
    }

    #[inline]
    pub fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
        self.total -= amount;
        debug_assert_eq!(self.total - self.held, self.available);
    }
}

use rust_decimal::Decimal;

#[derive(Clone, Copy, Debug, Default)]
pub struct Balance {
    pub available: Decimal,
    pub total: Decimal,
    pub held: Decimal,
}

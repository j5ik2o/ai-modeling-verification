pub mod adapters;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BillingResult {
    pub billed_energy_milli: u64,
    pub amount_yen: u64,
}

pub trait BillingSession {
    type Error;
    type ClosedSession: ClosedBillingSession<Error = Self::Error>;

    fn start(start_epoch_ms: i64, rate_yen_per_kwh: u32) -> Result<Self, Self::Error>
    where
        Self: Sized;

    fn bill_snapshot(
        &self,
        end_epoch_ms: i64,
        energy_milli: i64,
    ) -> Result<BillingResult, Self::Error>;

    fn stop(
        self,
        end_epoch_ms: i64,
        energy_milli: i64,
    ) -> Result<(BillingResult, Self::ClosedSession), Self::Error>;
}

pub trait ClosedBillingSession {
    type Error;

    fn bill_after_stop(
        &self,
        end_epoch_ms: i64,
        energy_milli: i64,
    ) -> Result<BillingResult, Self::Error>;
}

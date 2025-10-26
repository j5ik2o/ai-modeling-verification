pub(crate) const FREE_MINUTES: u128 = 5;
pub(crate) const MILLISECONDS_IN_MINUTE: u128 = 60_000;
/// 1 セッションあたりに許容する最大エネルギー量（1,000 kWh）。
pub(crate) const MAX_KWH_MILLI: u64 = 1_000_000;
/// 1 セッションあたりに許容する最大請求額（100万円）。
pub(crate) const MAX_YEN: u64 = 1_000_000;

mod base;
mod bill;
mod bounded;
mod charge_ratio;
mod chargeable_energy;
mod chargeable_window;
mod errors;
mod grace_period;
mod kwh_milli;
mod money_yen;
mod rate;
mod session_id;
mod timeline;

pub use base::Session;
pub use bill::SessionBill;
pub use bounded::BoundedU64;
pub use chargeable_energy::ChargeableEnergy;
pub use errors::SessionValueError;
pub use kwh_milli::KwhMilli;
pub use money_yen::MoneyYen;
pub use rate::RateYenPerKwh;
pub use session_id::SessionId;

#[cfg(test)]
mod tests;

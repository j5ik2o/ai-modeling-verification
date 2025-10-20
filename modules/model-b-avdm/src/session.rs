pub(crate) const FREE_MINUTES: u128 = 5;
pub(crate) const MILLISECONDS_IN_MINUTE: u128 = 60_000;
pub(crate) const FREE_MILLISECONDS: u128 = FREE_MINUTES * MILLISECONDS_IN_MINUTE;
/// 1 セッションあたりに許容する最大エネルギー量（1,000 kWh）。
pub(crate) const MAX_KWH_MILLI: u64 = 1_000_000;
/// 1 セッションあたりに許容する最大請求額（100万円）。
pub(crate) const MAX_YEN: u64 = 1_000_000;

mod energy;
mod errors;
mod money;
mod rate;
mod session_id;
mod base;

pub use energy::KwhMilli;
pub use errors::SessionValueError;
pub use money::MoneyYen;
pub use rate::RateYenPerKwh;
pub use session_id::SessionId;
pub use base::Session;

#[cfg(test)]
mod tests;

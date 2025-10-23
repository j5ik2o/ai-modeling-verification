use std::{
  convert::{From, TryFrom},
  num::NonZeroU32,
};

use super::{kwh_milli::KwhMilli, errors::SessionValueError, money_yen::MoneyYen};

/// kWh あたりの料金単価（円）を表す値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateYenPerKwh(pub(super) NonZeroU32);

impl RateYenPerKwh {
  /// 非ゼロの単価を生成する。
  #[must_use]
  pub fn new(value: NonZeroU32) -> Self {
    Self(value)
  }

  /// 生の値から単価を生成する。
  ///
  /// # Errors
  /// 0 以下の値が指定された場合、`SessionValueError::NonPositiveRate` を返します。
  pub fn try_new(value: u32) -> Result<Self, SessionValueError> {
    NonZeroU32::new(value)
      .map(Self::new)
      .ok_or(SessionValueError::NonPositiveRate)
  }

  /// エネルギー量に基づき金額を算出する。
  ///
  /// # Errors
  /// 算出結果が金額上限を超えた場合、`SessionValueError::AmountOutOfRange` を返します。
  ///
  /// # Returns
  /// 金額オブジェクトを `Ok` で返します。
  pub fn charge(self, billed_energy: KwhMilli) -> Result<MoneyYen, SessionValueError> {
    let billed_energy_milli = billed_energy.into_u128_milli();
    let rate_per_kwh = self.0.get() as u128;
    let amount = (billed_energy_milli * rate_per_kwh) / 1_000;
    MoneyYen::try_from_u128(amount)
  }
}

impl TryFrom<u32> for RateYenPerKwh {
  type Error = SessionValueError;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    Self::try_new(value)
  }
}

impl From<RateYenPerKwh> for u32 {
  fn from(value: RateYenPerKwh) -> Self {
    value.0.get()
  }
}

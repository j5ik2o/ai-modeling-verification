use std::convert::{From, TryFrom};

use super::{energy::KwhMilli, errors::SessionValueError, money::MoneyYen};

/// kWh あたりの料金単価（円）を表す値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateYenPerKwh(pub(super) u32);

impl RateYenPerKwh {
  /// 単価を生成する。
  ///
  /// # Errors
  /// 0 以下の値が指定された場合、`SessionValueError::NonPositiveRate` を返します。
  pub fn new(value: u32) -> Result<Self, SessionValueError> {
    if value == 0 {
      Err(SessionValueError::NonPositiveRate)
    } else {
      Ok(Self(value))
    }
  }

  /// エネルギー量に基づき金額を算出する。
  ///
  /// # Errors
  /// 算出結果が金額上限を超えた場合、`SessionValueError::AmountOutOfRange` を返します。
  pub fn charge(self, billed_energy: KwhMilli) -> Result<MoneyYen, SessionValueError> {
    let billed_energy_milli = billed_energy.into_u128_milli();
    let rate_per_kwh = u32::from(self) as u128;
    let amount = (billed_energy_milli * rate_per_kwh) / 1_000;
    MoneyYen::try_from_u128(amount)
  }
}

impl TryFrom<u32> for RateYenPerKwh {
  type Error = SessionValueError;

  fn try_from(value: u32) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<RateYenPerKwh> for u32 {
  fn from(value: RateYenPerKwh) -> Self {
    value.0
  }
}

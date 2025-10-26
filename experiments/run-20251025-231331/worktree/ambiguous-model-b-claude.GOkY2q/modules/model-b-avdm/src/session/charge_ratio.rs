use super::{errors::SessionValueError, kwh_milli::KwhMilli};

/// 課金対象時間と総時間の比率を表す値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChargeRatio {
  numerator:   u128,
  denominator: u128,
}

impl ChargeRatio {
  /// 課金窓の比率を生成する。
  ///
  /// # Errors
  /// 分母が 0 の場合はドメインエラーとして扱う。
  pub fn new(numerator: u128, denominator: u128) -> Result<Self, SessionValueError> {
    if denominator == 0 {
      return Err(SessionValueError::InvalidChargeRatio { numerator, denominator });
    }
    Ok(Self { numerator, denominator })
  }

  /// 0 かどうかを判定する。
  #[must_use]
  pub fn is_zero(&self) -> bool {
    self.numerator == 0
  }

  /// 比率をエネルギー量に適用し、課金対象エネルギーを求める。
  pub fn apply_to(&self, energy: KwhMilli) -> KwhMilli {
    if self.is_zero() {
      return KwhMilli::zero();
    }
    let energy_milli = energy.into_u128_milli();
    let billed_milli = (energy_milli * self.numerator) / self.denominator;
    KwhMilli::from_milli(billed_milli as u64)
  }

  /// 比率の分子を取得する。
  #[must_use]
  pub fn numerator(&self) -> u128 {
    self.numerator
  }

  /// 比率の分母を取得する。
  #[must_use]
  pub fn denominator(&self) -> u128 {
    self.denominator
  }
}

use std::convert::{From, TryFrom};

use super::{MAX_KWH_MILLI, bounded::BoundedU64, errors::SessionValueError};

/// エネルギー量（ミリkWh単位）を表す値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KwhMilli(pub(super) u64);

impl KwhMilli {
  /// 上限付きエネルギー量を生成する。
  #[must_use]
  pub fn new(value: BoundedU64<MAX_KWH_MILLI>) -> Self {
    Self(value.get())
  }

  /// 生の値からエネルギー量を生成する。
  ///
  /// # Errors
  /// 上限を超える値が渡された場合、`SessionValueError::EnergyOutOfRange` を返します。
  pub fn try_new(value: u64) -> Result<Self, SessionValueError> {
    let bounded = BoundedU64::<MAX_KWH_MILLI>::new(value)
      .ok_or(SessionValueError::EnergyOutOfRange { provided: value, max: MAX_KWH_MILLI })?;
    Ok(Self::new(bounded))
  }

  /// エネルギー量 0 を表す定数生成を行う。
  ///
  /// # Returns
  /// 0 を表す `KwhMilli`。
  pub fn zero() -> Self {
    Self(0)
  }

  pub(crate) fn from_milli(value: u64) -> Self {
    let bounded = BoundedU64::<MAX_KWH_MILLI>::new(value).expect("billed energy must be within total energy bounds");
    Self::new(bounded)
  }

  /// 符号付き整数からエネルギー量を生成する。
  ///
  /// # Errors
  /// 負の値、または上限を超える値が渡された場合は `SessionValueError` を返します。
  ///
  /// # Returns
  /// 妥当なエネルギー量を `Ok` で返します。
  pub fn try_from_i64(value: i64) -> Result<Self, SessionValueError> {
    if value < 0 {
      return Err(SessionValueError::NegativeEnergy { provided: value });
    }
    let unsigned = value as u64;
    Self::try_new(unsigned)
  }

  pub(crate) fn into_u128_milli(self) -> u128 {
    self.0 as u128
  }

  /// 上限を考慮した加算を行う。
  pub fn bounded_sum(self, other: Self) -> Result<Self, SessionValueError> {
    let total = self.0.saturating_add(other.0);
    Self::try_new(total)
  }
}

impl TryFrom<u64> for KwhMilli {
  type Error = SessionValueError;

  fn try_from(value: u64) -> Result<Self, Self::Error> {
    Self::try_new(value)
  }
}

impl From<KwhMilli> for u64 {
  fn from(value: KwhMilli) -> Self {
    value.0
  }
}

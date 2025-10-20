use std::convert::{From, TryFrom};

use super::{MAX_KWH_MILLI, errors::SessionValueError};

/// エネルギー量（ミリkWh単位）を表す値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KwhMilli(pub(super) u64);

impl KwhMilli {
  /// 0 以上かつ上限 `MAX_KWH_MILLI` 以下のエネルギー量（ミリkWh）を生成する。
  ///
  /// # Returns
  /// 妥当なエネルギー量を `Ok` で返します。
  ///
  /// # Errors
  /// 上限を超える値が渡された場合、`SessionValueError::EnergyOutOfRange` を返します。
  pub fn new(value: u64) -> Result<Self, SessionValueError> {
    if value > MAX_KWH_MILLI {
      Err(SessionValueError::EnergyOutOfRange {
        provided: value,
        max: MAX_KWH_MILLI,
      })
    } else {
      Ok(Self(value))
    }
  }

  /// エネルギー量 0 を表す定数生成を行う。
  ///
  /// # Returns
  /// 0 を表す `KwhMilli`。
  pub fn zero() -> Self {
    Self(0)
  }

  pub(crate) fn from_milli(value: u64) -> Self {
    Self(value)
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
    Self::new(unsigned)
  }

  pub(crate) fn into_u128_milli(self) -> u128 {
    self.0 as u128
  }
}

impl TryFrom<u64> for KwhMilli {
  type Error = SessionValueError;

  fn try_from(value: u64) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<KwhMilli> for u64 {
  fn from(value: KwhMilli) -> Self {
    value.0
  }
}

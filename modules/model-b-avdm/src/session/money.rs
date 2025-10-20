use std::convert::{From, TryFrom, TryInto};

use super::{MAX_YEN, errors::SessionValueError};

/// 料金の金額（円）を 0 以上の整数で保持するドメイン値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoneyYen(pub(super) u64);

impl MoneyYen {
  /// 0円以上 `MAX_YEN` 以下の金額を表す値オブジェクトを生成する。
  /// 日常的な課金規模を想定し、1,000,000 円（100万円）までを妥当範囲とする。
  ///
  /// # Errors
  /// 上限を超える金額が渡された場合、`SessionValueError::AmountOutOfRange` を返します。
  pub fn new(value: u64) -> Result<Self, SessionValueError> {
    if value > MAX_YEN {
      Err(SessionValueError::AmountOutOfRange {
        provided: value,
        max: MAX_YEN,
      })
    } else {
      Ok(Self(value))
    }
  }

  /// `u128` から金額を生成するヘルパー。
  ///
  /// # Errors
  /// 上限を超える値、または `u64` に収まらない場合は `SessionValueError` を返します。
  pub(crate) fn try_from_u128(value: u128) -> Result<Self, SessionValueError> {
    if value > MAX_YEN as u128 {
      Err(SessionValueError::AmountOutOfRange {
        provided: value as u64,
        max: MAX_YEN,
      })
    } else {
      value
        .try_into()
        .map(Self)
        .map_err(|_| SessionValueError::AmountOverflow { provided: value })
    }
  }
}

impl TryFrom<u64> for MoneyYen {
  type Error = SessionValueError;

  fn try_from(value: u64) -> Result<Self, Self::Error> {
    Self::new(value)
  }
}

impl From<MoneyYen> for u64 {
  fn from(value: MoneyYen) -> Self {
    value.0
  }
}

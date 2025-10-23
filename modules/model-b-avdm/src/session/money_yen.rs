use std::convert::{From, TryFrom, TryInto};

use super::{MAX_YEN, bounded::BoundedU64, errors::SessionValueError};

/// 料金の金額（円）を 0 以上の整数で保持するドメイン値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoneyYen(pub(super) u64);

impl MoneyYen {
  /// 上限付きの金額を生成する。
  #[must_use]
  pub fn new(value: BoundedU64<MAX_YEN>) -> Self {
    Self(value.get())
  }

  /// 生の値から金額を生成する。
  ///
  /// # Errors
  /// 上限を超える金額が渡された場合、`SessionValueError::AmountOutOfRange` を返します。
  pub fn try_new(value: u64) -> Result<Self, SessionValueError> {
    let bounded = BoundedU64::<MAX_YEN>::new(value)
      .ok_or(SessionValueError::AmountOutOfRange { provided: value, max: MAX_YEN })?;
    Ok(Self::new(bounded))
  }

  /// `u128` から金額を生成するヘルパー。
  ///
  /// # Errors
  /// 上限を超える値、または `u64` に収まらない場合は `SessionValueError` を返します。
  ///
  /// # Returns
  /// 妥当な金額を `Ok` で返します。
  pub(crate) fn try_from_u128(value: u128) -> Result<Self, SessionValueError> {
    if value > MAX_YEN as u128 {
      Err(SessionValueError::AmountOutOfRange { provided: value as u64, max: MAX_YEN })
    } else {
      let value_u64: u64 = value.try_into().map_err(|_| SessionValueError::AmountOverflow { provided: value })?;
      let bounded = BoundedU64::<MAX_YEN>::new(value_u64)
        .ok_or(SessionValueError::AmountOutOfRange { provided: value_u64, max: MAX_YEN })?;
      Ok(Self::new(bounded))
    }
  }
}

impl TryFrom<u64> for MoneyYen {
  type Error = SessionValueError;

  fn try_from(value: u64) -> Result<Self, Self::Error> {
    Self::try_new(value)
  }
}

impl From<MoneyYen> for u64 {
  fn from(value: MoneyYen) -> Self {
    value.0
  }
}

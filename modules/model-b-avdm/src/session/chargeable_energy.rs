use super::{chargeable_window::ChargeableWindow, errors::SessionValueError, kwh_milli::KwhMilli};

/// 課金対象となるエネルギー量を表現する値オブジェクト。
///
/// 元の総エネルギーと課金対象エネルギーの関係 (常に `billed <= total`)
/// を強制し、按分時の丸め方針を床に固定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChargeableEnergy {
  total:  KwhMilli,
  billed: KwhMilli,
}

impl ChargeableEnergy {
  /// 無課金を明示する。
  #[must_use]
  pub fn free(total: KwhMilli) -> Self {
    Self { total, billed: KwhMilli::zero() }
  }

  /// 総エネルギーと課金対象エネルギーのペアを生成する。
  ///
  /// # Errors
  /// `billed` が `total` を超えている場合、`SessionValueError::EnergyOutOfRange`
  /// を返します。
  pub fn new(total: KwhMilli, billed: KwhMilli) -> Result<Self, SessionValueError> {
    if u64::from(billed) > u64::from(total) {
      return Err(SessionValueError::EnergyOutOfRange { provided: u64::from(billed), max: u64::from(total) });
    }
    Ok(Self { total, billed })
  }

  /// 課金窓に基づきエネルギーを割り当てる。
  pub fn allocate(total_energy: KwhMilli, window: ChargeableWindow) -> Result<Self, SessionValueError> {
    window.allocate_energy(total_energy)
  }

  /// 課金対象エネルギー同士を合成する。
  pub fn combine(self, other: Self) -> Result<Self, SessionValueError> {
    let total = self.total.bounded_sum(other.total)?;
    let billed = self.billed.bounded_sum(other.billed)?;
    Self::new(total, billed)
  }

  /// 課金対象エネルギー量を返す。
  #[must_use]
  pub fn billable(self) -> KwhMilli {
    self.billed
  }

  /// セッションの総エネルギー量を返す。
  #[must_use]
  pub fn total_consumed(self) -> KwhMilli {
    self.total
  }
}

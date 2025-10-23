use super::{energy::KwhMilli, errors::SessionValueError};

/// 課金対象となるエネルギー量を表現する値オブジェクト。
///
/// 元の総エネルギーと課金対象エネルギーの関係 (常に `billed <= total`)
/// を強制し、按分時の丸め方針を床に固定する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChargeableEnergy {
    total: KwhMilli,
    billed: KwhMilli,
}

impl ChargeableEnergy {
    /// 総エネルギーと課金対象エネルギーのペアを生成する。
    ///
    /// # Errors
    /// `billed` が `total` を超えている場合、`SessionValueError::EnergyOutOfRange`
    /// を返します。
    pub fn new(total: KwhMilli, billed: KwhMilli) -> Result<Self, SessionValueError> {
        if u64::from(billed) > u64::from(total) {
            return Err(SessionValueError::EnergyOutOfRange {
                provided: u64::from(billed),
                max: u64::from(total),
            });
        }
        Ok(Self { total, billed })
    }

    /// 無料時間を差し引いた経過時間に基づいて課金対象エネルギーを算出する。
    ///
    /// 与えられた時間はミリ秒単位で解釈され、丸めは常に床となります。
    pub fn from_chargeable_window(
        total_energy: KwhMilli,
        chargeable_millis: u128,
        total_millis: u128,
    ) -> Result<Self, SessionValueError> {
        if total_millis == 0 {
            return Self::new(total_energy, KwhMilli::zero());
        }

        let effective_chargeable = chargeable_millis.min(total_millis);
        if effective_chargeable == 0 {
            return Self::new(total_energy, KwhMilli::zero());
        }

        let total_energy_milli = total_energy.into_u128_milli();
        let billed_milli = (total_energy_milli * effective_chargeable) / total_millis;
        let billed = KwhMilli::from_milli(billed_milli as u64);
        Self::new(total_energy, billed)
    }

    /// 課金対象エネルギー量を返す。
    pub fn billed(self) -> KwhMilli {
        self.billed
    }

    /// セッションの総エネルギー量を返す。
    pub fn total(self) -> KwhMilli {
        self.total
    }
}

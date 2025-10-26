use super::{
  charge_ratio::ChargeRatio, chargeable_energy::ChargeableEnergy, errors::SessionValueError, grace_period::GracePeriod,
  kwh_milli::KwhMilli,
};

/// 無料枠を差し引いた課金対象の時間窓。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChargeableWindow {
  chargeable_millis: u128,
  total_millis:      u128,
}

impl ChargeableWindow {
  /// 課金対象ウィンドウを生成する。
  #[must_use]
  pub fn new(chargeable_millis: u128, total_millis: u128) -> Self {
    Self { chargeable_millis, total_millis }
  }

  /// 無料ウィンドウを適用した結果を生成する。
  #[must_use]
  pub fn from_timeline(total_millis: u128, grace: GracePeriod) -> Self {
    let chargeable_millis = total_millis.saturating_sub(grace.millis());
    Self::new(chargeable_millis, total_millis)
  }

  /// 完全無料かどうかを判定する。
  #[must_use]
  pub fn is_free(&self) -> bool {
    self.chargeable_millis == 0
  }

  /// 課金対象時間の比率を取得する。
  pub fn ratio(&self) -> Result<ChargeRatio, SessionValueError> {
    ChargeRatio::new(self.chargeable_millis, self.total_millis)
  }

  /// 課金対象時間に基づいてエネルギーを割り当てる。
  pub fn allocate_energy(&self, total_energy: KwhMilli) -> Result<ChargeableEnergy, SessionValueError> {
    if self.total_millis == 0 || self.is_free() {
      return Ok(ChargeableEnergy::free(total_energy));
    }

    let ratio = self.ratio()?;
    let billed = ratio.apply_to(total_energy);
    ChargeableEnergy::new(total_energy, billed)
  }

  /// 課金対象となるミリ秒を返す。
  #[must_use]
  pub fn chargeable_millis(&self) -> u128 {
    self.chargeable_millis
  }

  /// 総ミリ秒を返す。
  #[must_use]
  pub fn total_millis(&self) -> u128 {
    self.total_millis
  }
}

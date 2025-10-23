use super::{
  chargeable_energy::ChargeableEnergy, errors::SessionValueError, kwh_milli::KwhMilli, money_yen::MoneyYen,
  rate::RateYenPerKwh,
};

/// セッション請求を表す値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionBill {
  energy: ChargeableEnergy,
  amount: MoneyYen,
}

impl SessionBill {
  /// 課金対象エネルギーと単価から請求を確定する。
  pub fn settle(energy: ChargeableEnergy, rate: RateYenPerKwh) -> Result<Self, SessionValueError> {
    let amount = rate.quote_for(energy.billable())?;
    Ok(Self { energy, amount })
  }

  /// 請求の合成を行う（同一セッション内の分割計算を想定）。
  pub fn merge(self, other: Self) -> Result<Self, SessionValueError> {
    let energy = self.energy.combine(other.energy)?;
    let amount = self.amount.saturating_add(other.amount)?;
    Ok(Self { energy, amount })
  }

  /// 課金対象エネルギーを返す。
  #[must_use]
  pub fn billable_energy(&self) -> KwhMilli {
    self.energy.billable()
  }

  /// 総エネルギー消費量を返す。
  #[must_use]
  pub fn total_energy(&self) -> KwhMilli {
    self.energy.total_consumed()
  }

  /// 請求金額を返す。
  #[must_use]
  pub fn amount_due(&self) -> MoneyYen {
    self.amount
  }
}

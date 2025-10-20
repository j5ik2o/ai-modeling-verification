use thiserror::Error;
use time::OffsetDateTime;

const FREE_MINUTES: u128 = 5;
const MILLISECONDS_IN_MINUTE: u128 = 60_000;
const FREE_MILLISECONDS: u128 = FREE_MINUTES * MILLISECONDS_IN_MINUTE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoneyYen(u64);

impl MoneyYen {
    /// 0円以上の金額を表す値オブジェクトを生成する。
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }

    fn try_from_u128(value: u128) -> Result<Self, SessionValueError> {
        value
            .try_into()
            .map(Self)
            .map_err(|_| SessionValueError::AmountOverflow { provided: value })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RateYenPerKwh(u32);

impl RateYenPerKwh {
    pub fn new(value: u32) -> Result<Self, SessionValueError> {
        if value == 0 {
            Err(SessionValueError::NonPositiveRate)
        } else {
            Ok(Self(value))
        }
    }

    pub fn value(self) -> u32 {
        self.0
    }

    pub fn charge(self, billed_energy: KwhMilli) -> Result<MoneyYen, SessionValueError> {
        let billed_energy_milli = billed_energy.value() as u128;
        let rate_per_kwh = self.value() as u128;
        let amount = (billed_energy_milli * rate_per_kwh) / 1_000;
        MoneyYen::try_from_u128(amount)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KwhMilli(u64);

impl KwhMilli {
    /// 0以上のエネルギー量（ミリkWh単位）を生成する。
    pub fn new(value: u64) -> Result<Self, SessionValueError> {
        Ok(Self(value))
    }

    pub fn zero() -> Self {
        Self(0)
    }

    pub(crate) fn from_milli(value: u64) -> Self {
        Self(value)
    }

    pub fn try_from_i64(value: i64) -> Result<Self, SessionValueError> {
        if value < 0 {
            Err(SessionValueError::NegativeEnergy { provided: value })
        } else {
            Ok(Self(value as u64))
        }
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionId(uuid::Uuid);

impl SessionId {
    pub fn new(id: uuid::Uuid) -> Self {
        Self(id)
    }

    pub fn value(self) -> uuid::Uuid {
        self.0
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SessionValueError {
    #[error("エネルギーは負にできません (入力: {provided})")]
    NegativeEnergy { provided: i64 },
    #[error("単価は1円/kWh以上である必要があります")]
    NonPositiveRate,
    #[error("終了時刻 {ended_at} は開始時刻 {started_at} より後でなければなりません")]
    InvalidTimeline {
        started_at: OffsetDateTime,
        ended_at: OffsetDateTime,
    },
    #[error("料金が表現できる上限を超過しました (入力: {provided})")]
    AmountOverflow { provided: u128 },
    #[error("セッション {session_id:?} は既に停止済みです")]
    AlreadyClosed { session_id: SessionId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveSession {
    id: SessionId,
    started_at: OffsetDateTime,
    rate_yen_per_kwh: RateYenPerKwh,
}

impl ActiveSession {
    pub fn new(id: SessionId, started_at: OffsetDateTime, rate_yen_per_kwh: RateYenPerKwh) -> Self {
        Self {
            id,
            started_at,
            rate_yen_per_kwh,
        }
    }

    #[allow(dead_code)]
    pub fn charge(&self, _energy: KwhMilli, _now: OffsetDateTime) -> MoneyYen {
        todo!()
    }

    pub fn stop(
        self,
        ended_at: OffsetDateTime,
        total_energy: KwhMilli,
    ) -> Result<ClosedSession, SessionValueError> {
        ClosedSession::new(self, ended_at, total_energy)
    }

    pub fn bill_snapshot(
        &self,
        ended_at: OffsetDateTime,
        total_energy: KwhMilli,
    ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
        let billed = self.billed_energy_for(ended_at, total_energy)?;
        let amount = self.rate_yen_per_kwh.charge(billed)?;
        Ok((billed, amount))
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn started_at(&self) -> OffsetDateTime {
        self.started_at
    }

    pub fn rate(&self) -> RateYenPerKwh {
        self.rate_yen_per_kwh
    }

    fn duration_millis(&self, ended_at: OffsetDateTime) -> Result<u128, SessionValueError> {
        let millis = (ended_at - self.started_at).whole_milliseconds();
        if millis <= 0 {
            Err(SessionValueError::InvalidTimeline {
                started_at: self.started_at,
                ended_at,
            })
        } else {
            Ok(millis as u128)
        }
    }

    fn billed_energy_for(
        &self,
        ended_at: OffsetDateTime,
        total_energy: KwhMilli,
    ) -> Result<KwhMilli, SessionValueError> {
        let total_ms = self.duration_millis(ended_at)?;
        let free_ms = FREE_MILLISECONDS.min(total_ms);
        let chargeable_ms = total_ms - free_ms;

        if chargeable_ms == 0 {
            return Ok(KwhMilli::zero());
        }

        let total_energy_milli = total_energy.value() as u128;
        let billed_energy_milli = (total_energy_milli * chargeable_ms) / total_ms;
        Ok(KwhMilli::from_milli(billed_energy_milli as u64))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClosedSession {
    id: SessionId,
    started_at: OffsetDateTime,
    ended_at: OffsetDateTime,
    rate_yen_per_kwh: RateYenPerKwh,
    total_energy: KwhMilli,
    billed_energy: KwhMilli,
    charged_amount: MoneyYen,
}

impl ClosedSession {
    pub fn new(
        active: ActiveSession,
        ended_at: OffsetDateTime,
        total_energy: KwhMilli,
    ) -> Result<Self, SessionValueError> {
        let (billed_energy, charged_amount) = active.bill_snapshot(ended_at, total_energy)?;

        Ok(Self {
            id: active.id,
            started_at: active.started_at,
            ended_at,
            rate_yen_per_kwh: active.rate_yen_per_kwh,
            total_energy,
            billed_energy,
            charged_amount,
        })
    }

    pub fn id(&self) -> SessionId {
        self.id
    }

    pub fn started_at(&self) -> OffsetDateTime {
        self.started_at
    }

    pub fn ended_at(&self) -> OffsetDateTime {
        self.ended_at
    }

    pub fn rate(&self) -> RateYenPerKwh {
        self.rate_yen_per_kwh
    }

    #[allow(dead_code)]
    pub fn total_energy(&self) -> KwhMilli {
        self.total_energy
    }

    pub fn billed_energy(&self) -> KwhMilli {
        self.billed_energy
    }

    pub fn charged_amount(&self) -> MoneyYen {
        self.charged_amount
    }

    pub fn bill_after_stop(
        &self,
        _ended_at: OffsetDateTime,
        _energy: KwhMilli,
    ) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
        Err(SessionValueError::AlreadyClosed {
            session_id: self.id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::Duration;

    fn ts(sec: i64) -> OffsetDateTime {
        OffsetDateTime::from_unix_timestamp(sec).expect("timestamp is valid")
    }

    #[test]
    fn kwh_milli_allows_zero_but_rejects_negative() {
        assert!(KwhMilli::new(0).is_ok());
        assert!(matches!(
            KwhMilli::try_from_i64(-1),
            Err(SessionValueError::NegativeEnergy { provided: -1 })
        ));
    }

    #[test]
    fn closed_session_requires_end_after_start() {
        let id = SessionId::new(uuid::Uuid::nil());
        let start = ts(0);
        let rate = RateYenPerKwh::new(10).expect("positive rate");
        let active = ActiveSession::new(id, start, rate);
        let end = ts(60);
        let energy = KwhMilli::new(1_000).expect("non-negative energy");
        let closed_via_stop = active
            .clone()
            .stop(end, energy)
            .expect("active session stops correctly");
        assert_eq!(closed_via_stop.ended_at(), end);
        assert_eq!(closed_via_stop.charged_amount().value(), 0);
        let closed = ClosedSession::new(active.clone(), end, energy)
            .expect("active session stops correctly");
        assert_eq!(closed.id().value(), uuid::Uuid::nil());
        assert_eq!(closed.started_at(), start);

        let active = ActiveSession::new(id, start, rate);
        let result = ClosedSession::new(active, start, energy);
        assert!(matches!(
            result,
            Err(SessionValueError::InvalidTimeline { started_at, ended_at })
                if started_at == start && ended_at == start
        ));
    }

    #[test]
    fn bill_snapshot_matches_stop() {
        let id = SessionId::new(uuid::Uuid::nil());
        let rate = RateYenPerKwh::new(50).expect("positive rate");
        let start = ts(0);
        let active = ActiveSession::new(id, start, rate);
        let end = ts(6 * 60);
        let total_energy = KwhMilli::new(2_400).expect("energy");
        let (billed, amount) = active
            .bill_snapshot(end, total_energy)
            .expect("snapshot works");
        assert_eq!(billed.value(), 400);
        assert_eq!(amount.value(), 20);
    }

    #[test]
    fn closed_session_rejects_additional_billing() {
        let id = SessionId::new(uuid::Uuid::nil());
        let rate = RateYenPerKwh::new(60).expect("positive rate");
        let start = ts(0);
        let end = ts(600);
        let energy = KwhMilli::new(2_000).expect("energy");
        let closed = ActiveSession::new(id, start, rate)
            .stop(end, energy)
            .expect("stop works");
        let err = closed
            .bill_after_stop(end + Duration::minutes(1), energy)
            .expect_err("billing after stop should fail");
        assert!(matches!(err, SessionValueError::AlreadyClosed { .. }));
    }
}

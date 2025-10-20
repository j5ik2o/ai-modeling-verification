use time::OffsetDateTime;

pub struct MoneyYen(pub u32);       // new()で非負保証
pub struct KwhMilli(pub u32);       // new()で >0 を保証

pub struct ActiveSession {
    started_at: OffsetDateTime,
    rate_yen_per_kwh: MoneyYen,
}
pub struct ClosedSession {
    started_at: OffsetDateTime,
    ended_at: OffsetDateTime,
    rate_yen_per_kwh: MoneyYen,
}

impl ActiveSession {
    pub fn charge(&self, energy: KwhMilli, now: OffsetDateTime) -> MoneyYen { todo!() }
    pub fn stop(self, ended_at: OffsetDateTime) -> ClosedSession { todo!() }
}
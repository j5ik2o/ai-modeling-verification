pub struct Session {
    pub started_at: Option<i64>,  // epoch millis
    pub ended_at: Option<i64>,
    pub kwh_milli: u64,
    pub rate_yen_per_kwh: u32,
    pub status: String,           // "active" / "closed" ← 文字列ゆれ地獄
}

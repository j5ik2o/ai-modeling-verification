use std::convert::TryFrom;

use time::OffsetDateTime;

use super::{chargeable_window::ChargeableWindow, errors::SessionValueError, grace_period::GracePeriod};

/// セッションの経過時間を表す値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SessionTimeline {
  elapsed_millis: u128,
}

impl SessionTimeline {
  /// 開始・終了時刻からタイムラインを構築する。
  pub fn between(started_at: OffsetDateTime, ended_at: OffsetDateTime) -> Result<Self, SessionValueError> {
    if ended_at <= started_at {
      return Err(SessionValueError::InvalidTimeline { started_at, ended_at });
    }

    let elapsed = ended_at - started_at;
    let millis = elapsed.whole_milliseconds();
    let elapsed_millis =
      u128::try_from(millis).map_err(|_| SessionValueError::InvalidTimeline { started_at, ended_at })?;

    Ok(Self { elapsed_millis })
  }

  /// 無料時間を適用し、課金ウィンドウを得る。
  #[must_use]
  pub fn consume_grace_period(&self, grace: GracePeriod) -> ChargeableWindow {
    ChargeableWindow::from_timeline(self.elapsed_millis, grace)
  }
}

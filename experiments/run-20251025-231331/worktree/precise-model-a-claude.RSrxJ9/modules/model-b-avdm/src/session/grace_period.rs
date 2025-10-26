use super::MILLISECONDS_IN_MINUTE;

/// 無料時間のウィンドウを表す値オブジェクト。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GracePeriod {
  millis: u128,
}

impl GracePeriod {
  /// ミリ秒単位で無料時間を生成する。
  pub const fn from_millis(millis: u128) -> Self {
    Self { millis }
  }

  /// 分単位で無料時間を生成する。
  pub const fn from_minutes(minutes: u128) -> Self {
    Self { millis: minutes * MILLISECONDS_IN_MINUTE }
  }

  /// 無料時間をミリ秒で返す。
  #[must_use]
  pub const fn millis(self) -> u128 {
    self.millis
  }

  /// 無料時間がゼロかどうかを判定する。
  #[must_use]
  pub const fn is_zero(self) -> bool {
    self.millis == 0
  }
}

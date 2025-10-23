/// 固定上限付きの `u64` 値を表すユーティリティ型。
///
/// コンストラクタで上限検証を行うことで、不正な範囲の値を静的に排除する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedU64<const MAX: u64>(u64);

impl<const MAX: u64> BoundedU64<MAX> {
  /// 上限 `MAX` 以下の値を生成する。
  ///
  /// # Returns
  /// `value <= MAX` のとき `Some`、それ以外は `None`。
  #[must_use]
  pub fn new(value: u64) -> Option<Self> {
    if value <= MAX {
      Some(Self(value))
    } else {
      None
    }
  }

  /// 内部の数値を取得する。
  #[must_use]
  pub fn get(self) -> u64 {
    self.0
  }
}

impl<const MAX: u64> From<BoundedU64<MAX>> for u64 {
  fn from(value: BoundedU64<MAX>) -> Self {
    value.get()
  }
}

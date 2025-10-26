/// セッション識別子を UUID で表す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionId(pub(super) uuid::Uuid);

impl SessionId {
  /// UUID から新しい `SessionId` を生成する。
  ///
  /// # Returns
  /// 生成された `SessionId`。
  pub fn new(id: uuid::Uuid) -> Self {
    Self(id)
  }

  /// 内部の UUID を取り出す。
  ///
  /// # Returns
  /// 保持している UUID。
  pub fn into_uuid(self) -> uuid::Uuid {
    self.0
  }

  /// セッションIDが nil かどうかを返す。
  ///
  /// # Returns
  /// nil であれば `true`。
  pub fn is_nil(&self) -> bool {
    self.0.is_nil()
  }
}

impl From<SessionId> for uuid::Uuid {
  fn from(value: SessionId) -> Self {
    value.0
  }
}

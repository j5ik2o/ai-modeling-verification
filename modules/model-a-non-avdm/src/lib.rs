//! model-a-non-avdm は、意図的に Tell Don't Ask を破る貧血モデルを通じて AVDM と非AVDM の違いを示す検証用クレートです。
#![deny(missing_docs)]
#![deny(clippy::missing_errors_doc)]
#![deny(clippy::missing_panics_doc)]
#![deny(clippy::missing_safety_doc)]
/// セッション生データと手続き的課金ロジックを扱うモジュール。
pub mod session;

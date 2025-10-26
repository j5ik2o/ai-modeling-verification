//! テストハーネスが利用するモデル別アダプタ群。

mod model_a;
mod model_b;

pub use model_a::{ClosedModelASession, ModelAError, ModelASession};
pub use model_b::{ClosedModelBSession, ModelBError, ModelBSession};

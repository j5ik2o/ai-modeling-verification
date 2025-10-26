# Repository Guidelines

このドキュメントは ai-modeling-verification に参加するコントリビュータ向けに、開発の踏襲手順と期待水準をまとめたガイドです。 初めての貢献でも迷わず着手できるよう、必須タスクと注意点を簡潔に記載しています。

## プロジェクト構成とモジュール構成
- ルートは Cargo ワークスペースで、modules/model-a-non-avdm と modules/model-b-avdm が対照的な実装を収めています。 共通の依存は Cargo.toml の [workspace.dependencies] で管理します。
- 料金計算ロジックは各モジュールの `src/session.rs` にあり、`main.rs` は CLI 実験のエントリとして保持してください。 補助資料は README.md と spec.md に集約されます。
- scripts/ は一時的なオートメーション用に確保されています。 target/ は生成物なのでコミット対象から除外し、不要なら `cargo clean` で削除してください。

## ビルド・テスト・開発コマンド
- `cargo build` : 全メンバーをコンパイルし、依存解決の問題を早期に検出します。
- `cargo run -p model-a-non-avdm` / `cargo run -p model-b-avdm` : それぞれのモデルを CLI で起動し、動作差分を再現します。
- `cargo fmt` / `cargo clippy --workspace --all-targets` : フォーマットと静的解析。 PR 前に必ず通し、警告は例外なく解消してください。
- `cargo test --workspace` : 全パッケージのテストを一括実行。 局所検証は `cargo test --package model-b-avdm` のように対象を絞ります。

## コーディングスタイルと命名規約
- Rust 2021 edition を採用し、4 スペースインデントと snake_case メソッド、UpperCamelCase 型、ALL_CAPS 定数を徹底します。
- ドメイン値は `MoneyYen` や `KwhMilli` のように newtype で表現し、検証ロジックをコンストラクタに集約します。 値オブジェクトを追加する際は不変条件を明示する失敗を返してください。
- コメントはビジネスルールと境界条件を説明し、`// TODO(issue-123):` 形式で追跡可能にします。

## テストガイドライン
- 単体テストは `src/session.rs` の `mod tests` に追加し、挙動差分は `tests/` ディレクトリで統合テスト化します。
- spec.md のシナリオを `#[test] fn scenario_01_free_five_minutes()` のように命名し、無料 5 分、按分計算、停止後の拒否を最低限カバーします。
- カバレッジ目標は料金計算と状態遷移の分岐 100% です。 time クレートを使うテストは `OffsetDateTime::parse` の固定入力で決定性を確保してください。

## コミットと PR ガイドライン
- コミットメッセージのサマリは命令形で 72 文字以内に収め、本文で設計意図と影響範囲を説明します。 関連 issue があれば `Refs #123` を追記してください。
- PR 説明には背景、変更点、確認手順、spec.md との対応状況を含めます。 画面出力がある場合はログまたはスクリーンショットを添付し、再現に必要な引数を共有します。
- 送信前に `cargo fmt`, `cargo clippy`, `cargo test` を連続実行し、CI 失敗を先回りで防いでください。 未対応の TODO は必ずコメントまたは別 issue でフォローします。

## セキュリティと設定メモ
- Rust stable の最新リリースでのビルドを前提とします。 未対応の新機能を使う場合は rust-toolchain.toml を追加し、チームに周知してください。
- 資料やログに含まれる顧客データやアクセストークンはリポジトリに入れないでください。 検証用の値は `fixtures/` など公開可能な場所に分離します。
- crates.io 以外の依存を追加するときはライセンスと更新方針を事前にレビューし、README.md にメモを残してください。

## 実験ログ
- パス = experiments
- `experiments/run-20251025-070615分をslides/slides.md` に反映する


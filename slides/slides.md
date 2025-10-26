---
theme: seriph
colorSchema: auto
class: text-center
highlighter: shiki
lineNumbers: false
info: AIモデリング検証
drawings:
  persist: false
transition: slide-left
title: AIモデリング検証
mdc: true
layout: cover
---

# AIモデリング検証

---

## アジェンダ

- 背景と目的
- 実験設計と計測方法
- 4実験の結果サマリ
- 失敗から学ぶ（ambiguous × model-a）
- 成功パターン（precise プロンプト）
- デモ運用と自動化の工夫
- 学び・次のアクション

---
layout: two-cols
---
## 背景と目的

- AIコーディング支援を利用しつつ、Always-Valid Domain Model(AVDM) が品質に与える影響を検証
- EV充電料金計算ドメインで、**曖昧 vs 精密**プロンプト、**非AVDM(model-a) vs AVDM(model-b)** の4通りを比較

::right::

![timeline](https://dummyimage.com/480x300/1f2933/ffffff&text=AI+Modeling+Verification)

---

## お題:EV充電料金の計算

- ユーザーが充電スタンドで `start` → `snapshot`（途中課金確認）→ `stop` する流れをモデル化
- 課金の基本式：**料金 = 単価（円/kWh） × 課金対象エネルギー**
- 主要ルール（spec.md のシナリオに対応）
  1. **無料5分（scenario1〜3,7）**：開始5分までに相当するエネルギーは請求対象から除外
  2. **按分は時間比例＆切り捨て（scenario4,12）**：エネルギーは時間で均等分布とみなし、小数は floor 処理
  3. **単調増加と決定性（scenario5,11）**：利用時間が伸びるほど料金・エネルギーは減らず、同じ入力なら同じ結果
  4. **停止後課金禁止（scenario6）**：`stop` 完了後に再課金しようとすると拒否
  5. **不正入力防御（scenario8〜10,14,15）**：ゼロ/負エネルギー、時間逆行、上限超過などはエラー
- これらを `Session` と `MoneyYen` などの値オブジェクトに閉じ込め、単体＋受入テストで検証

---

## プロンプト定義

- **明確なプロンプト**
    - 目的・変更範囲・ビジネスルール・テスト基準を明文化
    - 例: 「`modules/model-a-non-avdm/src/session.rs` の `stop` を修正し、受入テスト `scenario6` を通す。状態遷移図に合わせて Stop 後の二重課金を防ぐこと」
- **あいまいプロンプト**
  - タスク目標は示すが、変更対象や受け入れ条件が曖昧
  - 「料金計算を直してください」「失敗テストをとにかく通す」など抽象的な指示
  - LLM側の推測が増え、ドメイン制約を踏み外す危険が高い
- LLM の探索余地を適切に絞り、決定的な挙動と再現性を高める

---

## 実験設計

- `scripts/run-worktree-all.sh` が4ジョブを並列投入し、各ジョブは `tmp/worktrees/<prefix>-<model>-<mode>-XXXXXX` に作成した worktree で `claude` モードを駆動
- 実行ごとにログと PID を `tmp/logs/<timestamp>` に保存し、必要に応じて worktree を `tmp/worktrees/<timestamp>` に保持（今回のラン: `tmp/logs/run-20251025-231331` / `tmp/worktrees/run-20251025-231331`）
- 成果物：各ログ (`*.log`)、補助 PID (`*.pid`)、保存済み worktree、必要に応じてデモ動画
- 計測指標：テスト結果（単体・受入）、経過時間、代表エラーメッセージ

---

## 実験結果マトリクス

<small>データソース: `tmp/logs/run-20251025-231331`</small>

| プロンプト | モデル | 単体テスト | 受入テスト | 経過時間 |
|-------| --- | --- | --- | --- |
| 明確    | model-a | ✅ 8/8 | ✅ 9/9 | 02:14 |
| 明確    | model-b | ✅ 11/11 | ✅ 9/9 | 02:54 |
| あいまい  | model-a | ✅ 16/16 | ⚠️ 5/9 | 03:15 |
| あいまい  | model-b | ✅ 17/17 | ✅ 9/9 | 03:22 |

> ⚠️ `model-a billed energy mismatch` が scenario1/5/11/stop 検証で発生し、曖昧プロンプト × model-a が再び破綻。

<!-- Speaker Notes:
- 表の指標はすべて `tmp/logs/run-20251025-231331` の同一ランから抽出した値であると冒頭に念押しする
- 単体テスト件数は生成タスクに依存し、model-b では AVDM の防衛ロジック追加でテスト数も自然に増えたことを補足する
- 経過時間は `elapsed` ログをそのまま記載しており、プロンプト解像度とモデル選択が実際のリードタイムにどう響いたかを口頭で解説する
-->

---
layout: center
---

## あいまいプロンプト × 非AVDM 失敗の実相

```
assertion `left == right` failed: model-a billed energy mismatch for scenario1_six_minutes
  left: 2400
 right: 400
```

- 無料5分の按分ロジックが欠落し、6分目以降の課金量が過大になった（scenario1/5/11/stop が失敗）
- 時間が延びれば料金が増えるはず（単調増加）という前提や、同じ入力なら必ず同じ結果になる決定性（Determinism。参照透明性を成立させる前提の一つ）が壊れ、受入テスト 9 本のうち 4 本が連鎖的に失敗
- 生成コード: `tmp/worktrees/run-20251025-231331/ambiguous-model-a-claude.iNiQaQ/modules/model-a-non-avdm/src/session.rs:73-95` で無料時間を判定後も `session.billed_kwh_milli = session.kwh_milli;` と全量課金。FREE_DURATION 定数を持ちながら、時間按分が未実装のまま残った
```rust
if elapsed_millis <= FREE_DURATION_MILLIS {
    session.billed_kwh_milli = 0;
    // ...
    return Ok(0);
}

// 無料期間を差し引かず全量を課金対象に設定してしまう
session.billed_kwh_milli = session.kwh_milli;
```
<small>コード出典: `tmp/worktrees/run-20251025-231331/ambiguous-model-a-claude.iNiQaQ/modules/model-a-non-avdm/src/session.rs:73-95`</small>
- その結果、`stop_scenarios_match_expected`（backtrace: `spec-tests/tests/common.rs:104`）が想定値 400 ミリkWh に対し 2400 ミリkWh を返し、連鎖的に scenario5/11 と決定性検証が破綻
- 解析ログ: `tmp/logs/run-20251025-231331/ambiguous-a.log`
- **非AVDM** では例外を型で防げず、バグが再注入されやすい

<!-- Speaker Notes:
- このスライドでは scenario1/5/11/stop を例示しつつ、他の失敗も同じ按分欠落が原因だったと口頭で補足する
- `spec-tests/tests/common.rs:104` のアサーションがシナリオ期待値との突合せである点、ここからバグを掘った手順を説明する
- `ambiguous-a.log` には `Session already ended` 系の過去ログも残っており、曖昧プロンプトが同じ罠に繰り返し陥ったことを指摘する
-->

---
layout: center
---
## あいまいプロンプト × AVDM が踏みとどまった要因(1/2)

```text
test scenario5_progressive_billing_is_monotonic ... ok
test scenario6_rejects_billing_after_stop ... ok
test stop_scenarios_match_expected ... ok
```

- `SessionTimeline` や `ChargeableEnergy` など AVDM の値オブジェクトが不変条件を担保し、AI が生成した `Session` の変更でも型制約が破綻を防いだ
- 編集開始前に AI が値オブジェクト実装を読み込み（ログ冒頭: `tmp/logs/run-20251025-231331/ambiguous-b.log:10-60`）、不変条件を先に理解してから `Session` 実装へ着手
- <small>受入テスト `scenario10_invalid_timeline_is_rejected` / `scenario9_negative_energy_is_rejected` / `scenario15_energy_over_limit_is_rejected` が全て成功し、終了時刻逆転・負/過大エネルギーを自動的に拒否（ログ: `tmp/logs/run-20251025-231331/ambiguous-b.log:118-128`）</small>
- 受入テスト 9/9 合格、単体テストも完全成功。時間は要したが、ロジック破綻は発生せず（`elapsed: 03:22`）
- 解析ログ: `tmp/logs/run-20251025-231331/ambiguous-b.log`
 
<!-- Speaker Notes:
- 冒頭ログ（10-60 行）で VO 実装を読んでから修正に入った流れを説明し、曖昧プロンプトでも先にドメインルールを取り込めば成功率が上がると強調する
- `scenario10` `scenario9` `scenario15` が一発合格したのはタイムライン・エネルギー・金額 VO がそれぞれガードとして動いた結果だと口頭で補足する
- 右側ログは受入テストの成功抜粋なので、ここを指し示しながら QA チームとの共有方法を話す
-->

---

## あいまいプロンプト × AVDM が踏みとどまった要因(2/2)

```rust
fn compute_bill(...) -> Result<SessionBill, SessionValueError> {
  let timeline = SessionTimeline::between(started_at, ended_at)?;
  let window = timeline.consume_grace_period(Self::grace_period());
  let energy = ChargeableEnergy::allocate(total_energy, window)?;
  SessionBill::settle(energy, rate)
}
```

コード出典: `tmp/worktrees/run-20251025-231331/ambiguous-model-b-claude.GOkY2q/modules/model-b-avdm/src/session/base.rs:58-84`

- `consume_grace_period` が型レベルで無料5分を控除し、ChargeableEnergy が負値や超過量を拒否
- 生成コード: `tmp/worktrees/run-20251025-231331/ambiguous-model-b-claude.GOkY2q/modules/model-b-avdm/src/session/base.rs:58-84`


---
layout: two-cols
---

## 明確なプロンプトでの安定化

- 明確な仕様提示により、model-a でも期待通りの修正が入り 9/9 合格
- model-b では冗長な補強も入り、ドメインオブジェクトの自己診断が進化
- 所要時間が短縮され、ワークフロー全体の決定性が向上
- 解析ログ: `tmp/logs/run-20251025-231331/precise-a.log` / `tmp/logs/run-20251025-231331/precise-b.log`

::right::
```text
test result: ok. 9 passed; 0 failed; ... finished in 0.00s
elapsed: 02:14 (model-a precise)

test result: ok. 9 passed; 0 failed; ... finished in 0.00s
elapsed: 02:54 (model-b precise)
```

---

## デモとリスク管理

- 登壇当日は**録画済みデモ**を再生し、ライブ実行によるリスクを回避
- 最新ログ: `tmp/logs/run-20251025-231331`（必要に応じて `experiments/` へ昇格予定）

<video src="../experiments/run-20251025-070615/2025-10-25 07-06-22.mp4" controls style="width: 75%; margin: 1.5rem auto; border-radius: 12px; box-shadow: 0 10px 30px rgba(0,0,0,0.4);"></video>

- 再生前に `scripts/run-worktree-all.sh` の結果ログを提示し、信頼性の根拠を説明
- Slidev 上で動画が再生できない環境向けに、ダウンロードリンクを別途案内予定

---

## 学び

- **プロンプト解像度が最重要**: 曖昧指示では非AVDMがバグを再び生む
- **AVDM の防御力**: 値オブジェクトと不変条件で曖昧さを吸収
- **自動化の威力**: `run-worktree-all.sh` により、並列実験とログ収集が確実
- **記録の重要性**: 成果ログ＋動画で検証結果を「再演可能」に保管

---
layout: center
---

## ご清聴ありがとうございました

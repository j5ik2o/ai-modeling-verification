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

## 問題領域のざっくり像（EV充電）

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

## プロンプト定義（概要）

- **あいまいプロンプト**
  - タスク目標は示すが、変更対象や受け入れ条件が曖昧
  - 「料金計算を直してください」「失敗テストをとにかく通す」など抽象的な指示
  - LLM側の推測が増え、ドメイン制約を踏み外す危険が高い
- **明確なプロンプト**
  - 目的・変更範囲・ビジネスルール・テスト基準を明文化
  - 例: 「`modules/model-a-non-avdm/src/session.rs` の `stop` を修正し、受入テスト `scenario6` を通す。状態遷移図に合わせて Stop 後の二重課金を防ぐこと」
  - LLM の探索余地を適切に絞り、 determinism と再現性を高める

---

## 実験設計

- `scripts/run-worktree-all.sh` が4ジョブを並列投入し、各ジョブは一時 worktree で `claude` モードを駆動
- 実行ごとにログと PID を `${TMPDIR}` 配下に保存し、終了後に `experiments/run-20251025-070615/logs` へ収集
- 成果物：各ログ (`*.log`)、補助 PID (`*.pid`)、デモ動画 `2025-10-25 07-06-22.mp4`
- 計測指標：テスト結果（単体・受入）、経過時間、代表エラーメッセージ

---

## 実験結果マトリクス

| プロンプト | モデル | 単体テスト | 受入テスト | 経過時間 |
|-------| --- | --- | --- | --- |
| 明確    | model-a | ✅ 8/8 | ✅ 9/9 | 01:17 |
| 明確    | model-b | ✅ 19/19 | ✅ 9/9 | 04:18 |
| あいまい  | model-a | ✅ 8/8 | ⚠️ 4/9 | 03:14 |
| あいまい  | model-b | ✅ 18/18 | ✅ 9/9 | 06:13 |

> ⚠️ `Session already ended` が複数シナリオで発生し、ambiguous × model-a が失敗。

---
layout: center
---

## あいまいプロンプト × 非AVDM 失敗の実相

```
model-a stop failed for scenario1_six_minutes: 計算失敗: Session already ended
model-a snapshot failed at 3: 計算失敗: Session already ended
```

- あいまいプロンプトのため、停止処理前提が崩れたコードが生成され状態遷移が破綻
- 受入テスト 5/9 が崩壊し、停止後課金やモノトニック性が保証できなかった
- **非AVDM** では例外を型で防げず、バグが再注入されやすい

---
layout: two-cols
---
## あいまいプロンプト × AVDM が踏みとどまった要因

- AVDM により `Session` が不変条件を保持、曖昧指示でも破壊的変更が拒否
- 受入テスト 9/9 合格、単体テストも完全成功
- 時間は要したが、ロジック破綻は発生せず

::right::
```text
test scenario5_progressive_billing_is_monotonic ... ok
test scenario6_rejects_billing_after_stop ... ok
test stop_scenarios_match_expected ... ok
```

---
layout: two-cols
---

## 明確なプロンプトでの安定化

- 明確な仕様提示により、model-a でも期待通りの修正が入り 9/9 合格
- model-b では冗長な補強も入り、ドメインオブジェクトの自己診断が進化
- 所要時間が短縮され、ワークフロー全体の determinism が向上

::right::
```text
test result: ok. 9 passed; 0 failed
elapsed: 01:17 (model-a precise)

test result: ok. 9 passed; 0 failed
elapsed: 04:18 (model-b precise)
```

---

## デモとリスク管理

- 登壇当日は**録画済みデモ**を再生し、ライブ実行によるリスクを回避
- 動画: `experiments/run-20251025-070615/2025-10-25 07-06-22.mp4`

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

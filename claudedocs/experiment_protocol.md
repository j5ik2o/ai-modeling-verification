# 実験プロトコル - AVDM vs Non-AVDM AI実装比較

## 目的

曖昧な要件下でAIが実装する際、Always Valid Domain Model (AVDM) と非AVDMのどちらが高い成功率を示すかを検証する。

---

## 仮説

**H1: Model B (AVDM) の方がspec-testsの成功率が高い**

根拠:
- 型システムによる制約の明示化がAIのガイドレールになる
- 値オブジェクト（KwhMilli, MoneyYen等）から上限値を読み取れる
- 状態遷移が型で保証されている（Session::Active → Session::Closed）

**H2: Model A (Non-AVDM) では検証漏れが多発する**

根拠:
- 制約が暗黙的（コメントや定数に散在）
- 状態管理が手続き的（フラグベース）
- 検証ロジックを毎回書く必要がある

---

## 実験条件

### 統制条件（両モデルで同一）
- ✅ 同一の曖昧なプロンプト（日本語）
- ✅ 同一のテストハーネス（spec-tests）
- ✅ 完全に独立したClaude Codeセッション
- ✅ プロジェクトルートで作業（同じ作業ディレクトリ）
- ✅ spec.mdは参照可能（公開仕様書）

### 制約条件
- ❌ spec-tests/は参照禁止（後出しテスト）
- ❌ 他方のモデル実装は参照禁止
- ✅ 各モデルディレクトリ内のコードは参照可能

### 独立変数
- モデルタイプ（AVDM vs Non-AVDM）

### 従属変数（測定指標）
1. **spec-testsの成功率**（主要指標）
2. 実装時間
3. 参照したファイル数
4. AIの質問回数
5. 型エラー発生回数

---

## 実験手順

### 事前準備

#### 1. ベースライン確立
```bash
cd /Users/j5ik2o/Sources/ai-modeling-verification

# 現在のテスト状態確認
cargo test --workspace

# Model AとModel Bのユニットテストが通ることを確認
cargo test -p model-a-non-avdm
cargo test -p model-b-avdm
cargo test -p spec-tests
```

#### 2. 実装を初期化

**Model A:**
```bash
# modules/model-a-non-avdm/src/session.rs を編集
# calculate_charge() を todo!() に置き換え
# #[cfg(test)] セクションを削除
```

**Model B:**
```bash
# modules/model-b-avdm/src/session/base.rs を編集
# stop(), bill_snapshot(), bill_after_stop() 等を todo!() に置き換え
# テストを削除（該当ファイルの #[cfg(test)] セクション）
```

#### 3. 初期化状態をコミット
```bash
git add modules/model-a-non-avdm/src/session.rs
git add modules/model-b-avdm/src/session/base.rs
git commit -m "experiment: Initialize implementation for AI experiment

- Set calculate_charge() to todo!() in model-a
- Set Session methods to todo!() in model-b
- Remove all tests from both models
- Ready for AI implementation experiment"
```

---

### Phase 1: Model A実装

#### 1.1 セッション起動
```bash
# 新しいターミナル/Claude Codeウィンドウを開く
cd /Users/j5ik2o/Sources/ai-modeling-verification
```

#### 1.2 プロンプト投入
- ファイル: `claudedocs/prompt_model_a.md`
- 内容をそのままClaude Codeに投入
- **開始時刻を記録**

#### 1.3 実装観察（リアルタイム）

**観察シート（手動記録）:**
```markdown
## Model A 実装観察

### 開始時刻
[YYYY-MM-DD HH:MM:SS]

### 参照したファイル（順番に記録）
1.
2.
3.

### AIの質問内容
1.
2.

### AIの仮定・推測
1.
2.

### 警告チェック
- [ ] spec-tests/ を参照していないか？
- [ ] modules/model-b-avdm/ を参照していないか？

### 完了時刻
[YYYY-MM-DD HH:MM:SS]

### 所要時間
[XX分XX秒]
```

#### 1.4 実装完了後の検証
```bash
# 編集範囲確認
git status
git diff --name-only

# modules/model-a-non-avdm/ のみ変更されているか確認

# ユニットテスト実行
cargo test -p model-a-non-avdm

# 結果を記録
```

#### 1.5 コミット
```bash
git add modules/model-a-non-avdm/
git commit -m "experiment: AI implementation for Model A (non-AVDM)

実装者: Claude Code
セッション: 新規（独立）
制約: model-a-non-avdmのみ編集
参照: spec.md, model-a-non-avdm内部
開始: [時刻]
完了: [時刻]
所要時間: [XX分]
"
```

#### 1.6 セッション終了
- Claude Codeセッションを完全に終了
- ターミナルを閉じる
- **Model B実装前に十分な休憩を取る（脳のリセット）**

---

### Phase 2: Model B実装

#### 2.1 セッション起動（完全に新規）
```bash
# 新しいターミナル/Claude Codeウィンドウを開く（Phase 1とは別）
cd /Users/j5ik2o/Sources/ai-modeling-verification
```

#### 2.2 プロンプト投入
- ファイル: `claudedocs/prompt_model_b.md`
- 内容をそのままClaude Codeに投入
- **開始時刻を記録**

#### 2.3 実装観察（リアルタイム）

**観察シート（手動記録）:**
```markdown
## Model B 実装観察

### 開始時刻
[YYYY-MM-DD HH:MM:SS]

### 参照したファイル（順番に記録）
1.
2.
3.

### AIの質問内容
1.
2.

### AIの仮定・推測
1.
2.

### 警告チェック
- [ ] spec-tests/ を参照していないか？
- [ ] modules/model-a-non-avdm/ を参照していないか？

### 完了時刻
[YYYY-MM-DD HH:MM:SS]

### 所要時間
[XX分XX秒]
```

#### 2.4 実装完了後の検証
```bash
# 編集範囲確認
git status
git diff --name-only

# modules/model-b-avdm/ のみ変更されているか確認

# ユニットテスト実行
cargo test -p model-b-avdm

# 結果を記録
```

#### 2.5 コミット
```bash
git add modules/model-b-avdm/
git commit -m "experiment: AI implementation for Model B (AVDM)

実装者: Claude Code
セッション: 新規（独立）
制約: model-b-avdmのみ編集
参照: spec.md, model-b-avdm内部
開始: [時刻]
完了: [時刻]
所要時間: [XX分]
"
```

#### 2.6 セッション終了

---

### Phase 3: 評価

#### 3.1 Spec-Tests実行

```bash
cd /Users/j5ik2o/Sources/ai-modeling-verification

# Model A テスト実行
echo "=== Model A (Non-AVDM) Spec Tests ===" > results.txt
cargo test -p spec-tests --test acceptance_model_a -- --nocapture 2>&1 | tee -a results.txt

echo "" >> results.txt
echo "=== Model B (AVDM) Spec Tests ===" >> results.txt
cargo test -p spec-tests --test acceptance_model_b -- --nocapture 2>&1 | tee -a results.txt
```

#### 3.2 成功率計算

**Model A:**
```bash
grep "test result:" results.txt | head -1
# 例: test result: ok. 8 passed; 3 failed; 0 ignored; 0 measured; 0 filtered out
# 成功率: 8/11 = 72.7%
```

**Model B:**
```bash
grep "test result:" results.txt | tail -1
# 例: test result: ok. 10 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
# 成功率: 10/11 = 90.9%
```

#### 3.3 失敗パターン分析

各失敗テストについて分類：

**Model A 失敗分類:**
- [ ] scenario1-4: 基本ケース失敗（計算ロジックエラー）
- [ ] scenario5: 段階的課金の矛盾
- [ ] scenario6: 停止後課金の拒否失敗
- [ ] scenario9: 負値検証漏れ
- [ ] scenario10: タイムライン検証漏れ
- [ ] scenario11: 決定性違反
- [ ] scenario12: 端数処理エラー
- [ ] scenario14: 金額上限検証漏れ
- [ ] scenario15: エネルギー上限検証漏れ

**Model B 失敗分類:**
- [ ] 同上

---

## 結果記録フォーマット

### 定量データ

| 指標 | Model A (Non-AVDM) | Model B (AVDM) |
|------|-------------------|----------------|
| Spec-tests成功率 | X/11 (XX.X%) | Y/11 (YY.Y%) |
| 実装時間 | XX分XX秒 | YY分YY秒 |
| 参照ファイル数 | XX個 | YY個 |
| AI質問回数 | XX回 | YY回 |
| 編集ファイル数 | XX個 | YY個 |
| コード追加行数 | +XXX | +YYY |

### 定性データ

#### Model A 観察メモ
```
[AIの理解プロセス、仮定の正確性、困難だった点など]
```

#### Model B 観察メモ
```
[AIの理解プロセス、型ガイドの効果、スムーズだった点など]
```

---

## 成功基準

### 仮説H1の検証
- Model Bの成功率 > Model Aの成功率
- 差が10%以上あれば実質的に有意と判断

### 仮説H2の検証
- Model Aで scenario9, 14, 15（検証系）が失敗
- Model Bでは型システムにより自動的に防がれている

---

## リスク管理

### リスク1: 両モデルとも成功率が低い
**対策:**
- プロンプトの曖昧さを調整して再実験
- AIモデルのバージョンを記録

### リスク2: 両モデルとも成功率が高い
**対策:**
- より複雑なロジックで追加実験
- 実装時間・コード品質で比較

### リスク3: Model Aの方が成功率が高い
**対策:**
- 詳細分析（なぜそうなったか）
- Model Bのドメインモデルが過度に複雑だった可能性を検証

---

## 実験後の検証事項

### 妥当性チェック
- [ ] 両セッションが完全に独立していた
- [ ] spec-testsを参照していない
- [ ] プロンプトが同一（制約部分以外）
- [ ] 作業ディレクトリが同一

### データ整合性チェック
- [ ] git logで実装順序を確認
- [ ] コミットメッセージに時刻が記録されている
- [ ] 観察シートが完全に記入されている

---

## 次のステップ

1. **結果レポート作成**
   - `claudedocs/experiment_results.md`

2. **プレゼンテーション資料作成**
   - スライド: AVDMがAI時代に重要な理由

3. **ブログ記事執筆**
   - タイトル案: "AIが読みやすいコード = 人間も読みやすいコード"

4. **論文化の検討**
   - 学術的な貢献の可能性

---

## メタデータ

- **作成日**: 2025-10-21
- **プロトコルバージョン**: 1.0
- **使用AIモデル**: Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)
- **実験環境**: Claude Code CLI

# AI Modeling Verification Demo - 実行計画

## 目的

AVDM（Always Valid Domain Model）とNon-AVDMの実装難易度とAI支援効果を比較する。

## 仮説

**曖昧な要件下でも、AVDMは型システムによるガイドによりAIがより正確な実装を生成できる。**

---

## 事前準備

### 1. ベースライン確立

```bash
# 現在の状態を確認
cd /Users/j5ik2o/Sources/ai-modeling-verification
cargo test --workspace

# Model A, Model B両方のテストが通ることを確認
cargo test -p spec-tests
```

### 2. 実装の初期化

#### Model A (Non-AVDM)

```rust
// modules/model-a-non-avdm/src/session.rs
// calculate_charge関数を空にする

pub fn calculate_charge(session: &mut Session) -> Result<u32, String> {
  todo!("AIに実装させる")
}
```

```bash
# テストを削除
# modules/model-a-non-avdm/src/session.rsの#[cfg(test)]セクションを削除
```

#### Model B (AVDM)

```rust
// modules/model-b-avdm/src/session/base.rs
// 以下のメソッドを空にする

impl Session {
  pub fn stop(self, ended_at: OffsetDateTime, total_energy: KwhMilli)
      -> Result<Self, SessionValueError> {
    todo!("AIに実装させる")
  }

  pub fn bill_snapshot(&self, ended_at: OffsetDateTime, total_energy: KwhMilli)
      -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    todo!("AIに実装させる")
  }

  pub fn bill_after_stop(&self, ended_at: OffsetDateTime, total_energy: KwhMilli)
      -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    todo!("AIに実装させる")
  }

  fn bill_snapshot_for(...) -> Result<(KwhMilli, MoneyYen), SessionValueError> {
    todo!("AIに実装させる")
  }

  fn billed_energy_for(...) -> Result<KwhMilli, SessionValueError> {
    todo!("AIに実装させる")
  }

  fn duration_millis(...) -> Result<u128, SessionValueError> {
    todo!("AIに実装させる")
  }
}
```

```bash
# テストを削除（該当ファイルのテストセクション）
```

### 3. 不明確プロンプトの準備

```markdown
あなたは充電セッションの課金ロジックを実装するRustエンジニアです。

既存のSession型に対して課金計算を実装してください。
使えるドメインモデルの実装はすでにあるので、それらを使ってください。
プロンプトと既存コードだけを見て作業してください。

## やりたいこと

EV充電セッションの課金処理を仕上げたいです。
開始したら途中で様子を見ることもあるし、最後に止めて確定請求したいこともあります。
短時間のサービス枠があったり、現実とかけ離れた入力は受け付けないようにしたいので、
そのあたりは常識的に整えておいてください。

テストでは、日常的な利用パターンから不正っぽい操作までいくつか押さえておきたいです。
たとえば：
- 開始直後にちょっとエネルギーが増えただけなら無料扱いになる
- 止めたあとにまた請求しようとしたら断る
- 桁外れのエネルギーや金額になりそうなら弾く
- 決定性（同じ条件なら同じ結果）も大事

細かい条件は明文化しきれませんが、後から追加で境界ケースを突っ込まれても
破綻しない造りとテストをお願いします。
```

---

## 実験実施

### Phase 1: Model A実装

1. **新しいClaude Code セッション起動**
   ```bash
   # プロジェクトルートで作業
   cd /Users/j5ik2o/Sources/ai-modeling-verification
   ```

2. **プロンプト投入**（制約付き不明確プロンプト）
   ```markdown
   # 重要な制約
   - **modules/model-a-non-avdm/ のみを編集してください**
   - modules/model-b-avdm/ は見ないでください（別実装）
   - spec-tests/ は見ないでください（後で実行するテスト）
   - spec.md は参照OKです（仕様書）

   [不明確プロンプトの内容...]
   ```

3. **実装を観察・監視**
   - AIがどのような質問をするか
   - どのコードを参照するか → ⚠️ model-b-avdm/を見ていないか監視
   - どのような仮定をするか
   - 実装にかかる時間

4. **実装完了後、検証とコミット**
   ```bash
   # 編集ファイル確認（model-a以外を触っていないか）
   git status

   # コミット
   git add modules/model-a-non-avdm/
   git commit -m "feat: AI implementation for Model A (non-AVDM)

   実装者: Claude Code (新セッション)
   制約: model-a-non-avdmのみ編集
   参照: spec.md, model-a-non-avdm内部"
   ```

5. **セッション終了**（重要！完全にクリーンな状態にする）

### Phase 2: Model B実装

1. **完全に新しいClaude Code セッション起動**（重要！）
   ```bash
   # プロジェクトルートで作業（同じパス）
   cd /Users/j5ik2o/Sources/ai-modeling-verification
   ```

2. **同一プロンプト投入**（制約のみ変更）
   ```markdown
   # 重要な制約
   - **modules/model-b-avdm/ のみを編集してください**
   - modules/model-a-non-avdm/ は見ないでください（別実装）
   - spec-tests/ は見ないでください（後で実行するテスト）
   - spec.md は参照OKです（仕様書）

   [同じ不明確プロンプトの内容...]
   ```

3. **実装を観察・監視**（同じ観察ポイント + 制約遵守確認）
   - ⚠️ model-a-non-avdm/を見ていないか監視

4. **実装完了後、検証とコミット**
   ```bash
   # 編集ファイル確認（model-b以外を触っていないか）
   git status

   # コミット
   git add modules/model-b-avdm/
   git commit -m "feat: AI implementation for Model B (AVDM)

   実装者: Claude Code (新セッション)
   制約: model-b-avdmのみ編集
   参照: spec.md, model-b-avdm内部"
   ```

5. **セッション終了**

### Phase 3: 評価

#### 3.1 Spec-Tests実行

```bash
cd /Users/j5ik2o/Sources/ai-modeling-verification

# Model A テスト
cargo test -p spec-tests --test acceptance_model_a -- --nocapture

# Model B テスト
cargo test -p spec-tests --test acceptance_model_b -- --nocapture
```

#### 3.2 成功率の計算

```bash
# テスト結果をファイルに保存
cargo test -p spec-tests --test acceptance_model_a -- --nocapture > model_a_results.txt 2>&1
cargo test -p spec-tests --test acceptance_model_b -- --nocapture > model_b_results.txt 2>&1

# 成功・失敗カウント
grep -E "test result:" model_a_results.txt
grep -E "test result:" model_b_results.txt
```

---

## 評価指標

### 定量指標

| 指標 | Model A | Model B |
|------|---------|---------|
| テスト成功率 | ?/11 (??%) | ?/11 (??%) |
| 実装時間 | ??分 | ??分 |
| AI質問回数 | ?? | ?? |
| コード行数 | ?? | ?? |
| 型エラー回数 | ?? | ?? |

### 定性指標

| 観察項目 | Model A | Model B |
|----------|---------|---------|
| AIの理解速度 | | |
| 仮定の正確性 | | |
| エラーハンドリング | | |
| 境界値テスト網羅性 | | |
| コードの可読性 | | |

### 失敗パターン分類

**成功したテストケース：**
- [ ] scenario1: 6分で1分課金
- [ ] scenario2: 4分無料
- [ ] scenario3: 5分無料
- [ ] scenario5: 段階的課金
- [ ] scenario6: 停止後拒否
- [ ] scenario9: 負値拒否
- [ ] scenario10: 逆転タイムライン拒否
- [ ] scenario11: 決定性
- [ ] scenario12: 端数処理と単調性
- [ ] scenario14: 上限超過拒否（金額）
- [ ] scenario15: 上限超過拒否（エネルギー）

**失敗の分類：**
- 境界値エラー（off-by-one等）
- 検証漏れ（不正入力を受け入れた）
- 計算ロジックエラー（按分計算の誤り）
- 状態管理エラー（再課金防止の失敗）
- 型の不一致

---

## 期待される結果

### 仮説1: Model Bの成功率が高い

**理由:**
- 型定義が制約を明示（`KwhMilli`, `MoneyYen`, `RateYenPerKwh`）
- 状態遷移が型で保証（`Session::Active` → `Session::Closed`）
- エラー型が境界条件を列挙（`SessionValueError`）
- AIが既存の値オブジェクトから上限値を読み取れる

### 仮説2: Model Aでよくある失敗パターン

1. **検証漏れ**
   - 負値チェックを忘れる
   - 上限チェックを忘れる（または間違った値を使う）

2. **状態管理ミス**
   - `already_billed`フラグの更新漏れ
   - `status`チェックの条件ミス

3. **計算ロジックエラー**
   - 無料時間の按分計算ミス
   - 切り捨て処理の実装ミス

### 仮説3: Model Bでも起こりうる失敗

1. **ドメインモデルの誤解**
   - 既存の型を使わずに独自実装してしまう

2. **計算ロジックの複雑性**
   - 按分計算自体は型では保証できない

---

## デモプレゼンテーション構成

### 1. 問題提起 (2分)
「曖昧な要件でもAIは正しく実装できるか？」

### 2. 実験デザイン (3分)
- 2つのモデル（AVDM vs Non-AVDM）
- 同一の曖昧プロンプト
- 後出しの包括的テスト

### 3. 実装プロセスの比較 (5分)
- AIの理解プロセスの違い
- 参照したコードの違い
- 仮定の正確性の違い

### 4. 結果発表 (3分)
- テスト成功率の比較
- 失敗パターンの分類
- 統計的有意性

### 5. 考察 (5分)
- なぜAVDMが優位だったのか
- 型システムがAIのガイドレールになる
- 実務への示唆

### 6. まとめ (2分)
「Always Valid Domain ModelはAI時代にこそ重要」

---

## リスクと対策

### リスク1: 両モデルとも成功率が低い

**対策:**
- AIの能力不足ではなく、プロンプトの問題の可能性
- プロンプトを段階的に明確化して再実験

### リスク2: 両モデルとも成功率が高い

**対策:**
- より複雑なロジックで再実験
- 実装時間やコード品質で比較

### リスク3: Model Aの方が成功率が高い

**分析:**
- なぜそうなったか詳細調査
- Model Bのドメインモデルが逆に複雑すぎた可能性
- 学びとして共有

---

## 次のステップ

1. **実験実施**（このドキュメントに従って）
2. **結果の詳細分析**
3. **ブログ記事化**
4. **技術カンファレンスでの発表**
5. **論文化の検討**

---

## メタデータ

- 作成日: 2025-10-21
- 実験者: AI Modeling Verification Project
- 推定所要時間: 3-4時間（実装 + 評価）

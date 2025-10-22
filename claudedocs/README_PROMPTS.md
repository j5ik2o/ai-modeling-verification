# 実験用プロンプトファイル一覧

## 📁 ファイル構成

```
claudedocs/
├── README_PROMPTS.md                    # このファイル
├── experiment_protocol.md               # 実験手順書（全体の流れ）
├── demo_execution_plan.md               # デモ実行計画
│
├── prompt_model_a_ambiguous.md          # Model A: 不明確版プロンプト
├── prompt_model_a_precise.md            # Model A: 正確版プロンプト
│
├── prompt_model_b_ambiguous.md          # Model B: 不明確版プロンプト
└── prompt_model_b_precise.md            # Model B: 正確版プロンプト
```

---

## 🎯 プロンプトの種類と用途

### 1. 不明確版プロンプト（Ambiguous）

**目的**: 実務でよくある曖昧な要件下でのAI実装能力を測定

**特徴**:
- ✅ 要件が意図的に曖昧
- ✅ 「常識的に」「そのへんは」など抽象的な表現
- ✅ 具体的な数値は明示しない（5分、上限値等）
- ✅ テストケースも例示レベル

**使用シーン**:
- 主実験（AVDM vs Non-AVDMの成功率比較）
- 「AIが型システムからどれだけ学習できるか」を検証

**ファイル**:
- `prompt_model_a_ambiguous.md`
- `prompt_model_b_ambiguous.md`

---

### 2. 正確版プロンプト（Precise）

**目的**: 詳細な仕様が与えられた場合のAI実装品質を測定

**特徴**:
- ✅ spec.md準拠の詳細仕様
- ✅ 具体的な数値（5分=300,000ms、上限1,000,000等）
- ✅ 計算式の明示
- ✅ 完全なテストケース

**使用シーン**:
- 対照実験（不明確版との比較）
- ベースライン確立（両モデルの理想的な成功率）
- 「仕様が明確ならどちらも成功する」ことの確認

**ファイル**:
- `prompt_model_a_precise.md`
- `prompt_model_b_precise.md`

---

## 🧪 実験マトリクス

| 実験 | Model A | Model B | 目的 |
|------|---------|---------|------|
| **実験1: 不明確版** | `prompt_model_a_ambiguous.md` | `prompt_model_b_ambiguous.md` | **主実験**: AVDMの優位性検証 |
| **実験2: 正確版** | `prompt_model_a_precise.md` | `prompt_model_b_precise.md` | 対照実験: ベースライン確立 |
| **実験3: 交差検証** | 不明確 → 正確 | 不明確 → 正確 | 段階的改善の検証 |

---

## 📊 期待される結果

### 実験1: 不明確版（主実験）

```
仮説:
  Model B成功率 > Model A成功率 + 10%

予測:
  Model A: 60-70% (7-8/11)
  Model B: 80-90% (9-10/11)

理由:
  - Model Bは型定義から制約を読み取れる
  - Model Aは定数を見逃す可能性が高い
```

### 実験2: 正確版（対照実験）

```
仮説:
  両モデルとも高成功率（90%以上）

予測:
  Model A: 90-100% (10-11/11)
  Model B: 90-100% (10-11/11)

理由:
  - 仕様が明確なら実装差は小さい
  - 計算ロジックの複雑性のみが残る
```

---

## 🚀 使用方法

### Step 1: 実験選択

どの実験を行うか決定：
- **主実験**: 不明確版（AVDM優位性の検証）
- **対照実験**: 正確版（ベースライン確立）
- **両方**: 完全な比較

### Step 2: プロンプトファイル選択

```bash
# 例: 主実験（不明確版）でModel A実装
cat claudedocs/prompt_model_a_ambiguous.md
# 内容をClaude Codeにコピー&ペースト

# 例: 対照実験（正確版）でModel B実装
cat claudedocs/prompt_model_b_precise.md
# 内容をClaude Codeにコピー&ペースト
```

### Step 3: 実験プロトコル遵守

`experiment_protocol.md` に従って：
1. セッション起動
2. プロンプト投入
3. 実装観察
4. 結果記録

---

## 📝 プロンプト間の差分

### Model A: ambiguous vs precise

**共通点**:
- 制約条件（spec-tests/禁止等）
- 実装対象（`calculate_charge()`）
- 実験記録欄

**相違点**:

| 項目 | Ambiguous | Precise |
|------|-----------|---------|
| 無料時間 | 「短時間のサービス枠」 | 「5分間（300,000ミリ秒）」 |
| 上限 | 「桁外れは弾く」 | 「1,000,000ミリkWh」 |
| 計算式 | 「按分計算」 | `floor((energy × ratio))` |
| テスト | 例示レベル | 完全なコード |

### Model B: ambiguous vs precise

**共通点**:
- Model Aと同様

**相違点**:
- Model Aと同じ差分
- 加えて「既存型を活用」の指示が正確版では具体的

---

## 🔬 実験結果の解釈

### パターン1: 不明確版でModel B優位

```
→ 仮説H1支持: AVDMは型ガイドにより曖昧さを補完できる
```

### パターン2: 両バージョンで両モデル同等

```
→ 型システムの効果は限定的、または仕様の明確さが支配的
```

### パターン3: 正確版で両モデル成功、不明確版で両モデル失敗

```
→ プロンプトの曖昧さが支配的、型システムの効果は測定できず
```

### パターン4: Model A優位（予想外）

```
→ 詳細分析が必要: Model Bのドメインモデルが過度に複雑？
```

---

## 📌 重要な注意事項

### 1. セッション独立性

```bash
# ✅ 正しい
Session 1: Model A (ambiguous)
  → 完全終了
Session 2: Model B (ambiguous)  # 新規セッション

# ❌ 間違い
Session 1: Model A (ambiguous)
  → 継続
  → Model B (ambiguous)  # 同じセッション
```

### 2. プロンプトの完全性

- ファイル全体をコピー&ペースト
- 手動編集しない
- 制約条件を必ず含める

### 3. 観察の徹底

- AIが参照したファイルを記録
- 質問内容を記録
- 制約違反を即座に指摘

---

## 🔄 実験の再現性

### 再現に必要な情報

1. **使用したプロンプトファイル名**
   - 例: `prompt_model_a_ambiguous.md`

2. **Claude Codeのバージョン**
   - 例: Claude Sonnet 4.5 (claude-sonnet-4-5-20250929)

3. **実施日時**
   - 例: 2025-10-21 14:30 JST

4. **観察記録**
   - プロンプトファイル内の実験記録欄に記入

### 他の研究者による再現

```bash
# 1. リポジトリクローン
git clone https://github.com/[user]/ai-modeling-verification.git
cd ai-modeling-verification

# 2. 特定コミットにチェックアウト（実験時点）
git checkout [commit-hash]

# 3. プロンプトファイル使用
cat claudedocs/prompt_model_a_ambiguous.md
# Claude Codeに投入

# 4. 結果比較
cargo test -p spec-tests
```

---

## 📚 関連ドキュメント

- **`experiment_protocol.md`**: 詳細な実験手順
- **`demo_execution_plan.md`**: デモ実行計画
- **`spec.md`**: 仕様書（参照可能）
- **`spec-tests/`**: 受け入れテスト（参照禁止）

---

## 🎓 学術的価値

この実験デザインは以下の研究質問に答えます：

1. **RQ1**: 曖昧な要件下で、型システムはAIのガイドレールとして機能するか？
2. **RQ2**: AVDM採用による実装成功率の向上は定量的に測定可能か？
3. **RQ3**: 仕様の明確さと型システムの効果の相互作用はどうか？

**測定指標**:
- 主要: spec-testsの成功率
- 副次: 実装時間、コード品質、エラー発生率

**貢献**:
- AI時代のドメインモデリング手法への示唆
- 型システムの教育効果の定量化
- DDDコミュニティへの新しい視点

---

## メタデータ

- **作成日**: 2025-10-21
- **最終更新**: 2025-10-21
- **バージョン**: 1.0
- **管理者**: AI Modeling Verification Project

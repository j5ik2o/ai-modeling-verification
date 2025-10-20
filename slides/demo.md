---
theme: default
layout: cover
title: "AVDM vs 非AVDM"
subtitle: "曖昧なプロンプトに対する AI コーディング挙動の比較"
author: ""
---

---
layout: section
title: ゴール
---

- 曖昧なプロンプトが生むリスクと、設計で防げる範囲を確認する
- Always-Valid Domain Model (AVDM) と非AVDMの実装を比較する
- spec-tests を使い、同じ仕様で両モデルの挙動をデモする

---
layout: section
title: 仕様
---

- ルールはたった二つ
  1. **最初の5分は無料**
  2. **停止後は課金不可**
- 曖昧な指示：「5分無料で途中料金も見せて」…拒否せず実装できるか？

---
layout: section
title: リポジトリ構成
---

```
modules/
  model-a-non-avdm/  # 貧血モデル (データ構造 + 外部処理)
  model-b-avdm/      # AVDM (値オブジェクト + Session enum)
spec-tests/          # 共通受入テスト (BillingSession トレイト)
```

- **非AVDM**: `Session` はデータ袋、`calculate_charge` が手続き的
- **AVDM**: `MoneyYen`, `KwhMilli`, `Session` が不変条件をカプセル化

---
layout: section
title: デモ進行
---

1. 導入（1分）
2. 公平プロンプト → 非AVDM → `cargo test`
3. 公平プロンプト → AVDM → `cargo test`
4. 曖昧プロンプト → 非AVDM → `cargo test`
5. 曖昧プロンプト → AVDM → `cargo test`
6. まとめ（1分）

---
layout: section
title: 公平プロンプト例
---

```
modules/model-a-non-avdm の session.rs の calculate_charge が
「最初の5分無料」「停止後課金不可」を満たすよう調整してください。
既存のシナリオ1〜3が通ることを確認してください。
```

- 非AVDMでも丁寧な指示なら緑
- `cargo test --package model-a-non-avdm`

---
layout: section
title: 公平プロンプトの AV 側
---

```
modules/model-b-avdm の Session に対して同じ仕様を確認し、
必要なら補強してください。シナリオ1〜4を通してください。
```

- AVDMも同じ条件で緑
- `cargo test --package model-b-avdm`

---
layout: section
title: 曖昧プロンプト例
---

```
充電セッションの料金を調整してください。
最初の5分無料、途中で料金を表示できるようにしておいてください。
停止後の扱いは任せます。
```

- 非AVDM → `cargo test --package spec-tests`
  - scenario5/6が赤になりやすい（停止後課金・if抜け）
- AVDM → 同プロンプトでも緑（型と値オブジェクトがガイド）

---
layout: section
title: spec-tests
---

- `spec-tests/tests/acceptance.rs` に scenario1〜12
- 両モデルを `BillingSession` 経由で共通テスト
- 曖昧なプロンプトでも AVDM 側は `SessionValueError` で守られる

---
layout: image
title: Tell Don't Ask
image: https://raw.githubusercontent.com/microsoft/presidio/main/docs/images/tell-dont-ask.png
backgroundSize: contain
---

- 非AVDM: `Session` がデータ袋 → 呼び出し側で判断
- AVDM: `Session::stop`, `RateYenPerKwh::new` が検証を内包

---
layout: section
title: まとめ
---

- **曖昧な指示 = 推測の余地** → 非AVDMは割れ窓化しやすい
- **AVDMは型が文法を与える** → 「できること」が限定され、誤りを防ぐ
- AI活用時も、プロンプト精度に頼らず設計で安全弁を用意する

---
layout: section
title: 参考コマンド
---

- `cargo test --package model-a-non-avdm`
- `cargo test --package model-b-avdm`
- `cargo test --package spec-tests`
- `cargo make clippy`


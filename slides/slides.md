---
theme: seriph
colorSchema: dark
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

## 目的

- Always-Valid Domain ModelがどのぐらいAIを支援するのか調べる

---

## 調査方法

- EVの充電料金を計算する問題



---


## あいまいAパターンの受入テストの結果

3分で作業完了したが、5/9で失敗した。

```
failures:
    scenario11_same_input_same_result
    scenario12_rounding_is_floor_and_monotonic
    scenario5_progressive_billing_is_monotonic
    scenario6_rejects_billing_after_stop
    stop_scenarios_match_expected

test result: FAILED. 4 passed; 5 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
error: test failed, to rerun pass `-p spec-tests --test acceptance_model_a`
cargo test failed for package spec-tests (continuing)
elapsed: 03:14
```

---

## あいまいBパターンの単体テストの結果

15個ある単体テストのうち1個が失敗。

```text
Finished `test` profile [unoptimized + debuginfo] target(s) in 1.00s
Running unittests src/lib.rs (target/debug/deps/model_b_avdm-fab8b06ee93ef29f)
error: test failed, to rerun pass `-p model-b-avdm --lib`

running 18 tests
test session::tests::test_bill_after_stop_on_closed_session ... ok
<snip>
test session::tests::test_free_period_just_over_5_minutes ... FAILED
<snip>
test result: FAILED. 17 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

# あいまいBパターンの受入テストの結果

6分ぐらいかかったが、最終的にすべての受入テストがパスした。

```
running 9 tests
test scenario11_same_input_same_result ... ok
test scenario10_invalid_timeline_is_rejected ... ok
test scenario14_amount_over_limit_is_rejected ... ok
test scenario12_rounding_is_floor_and_monotonic ... ok
test scenario15_energy_over_limit_is_rejected ... ok
test scenario6_rejects_billing_after_stop ... ok
test scenario5_progressive_billing_is_monotonic ... ok
test scenario9_negative_energy_is_rejected ... ok
test stop_scenarios_match_expected ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

elapsed: 06:13
```

---

## 学びとアンチパターン
- ドメインイベントが定義されていないと例外処理が暗黙になる
- データサイエンスチームとドメイン専門家の言語断絶を放置しない
- メトリクスだけで判断せずユーザーストーリーを回帰対象に含める

---

## 次のアクション
- 今週中に現状モデルのドメインイベント棚卸しを実施
- 主要境界シナリオを例ベーステストとして自動化
- モデルリリース前の検証プレイブックを整備しナレッジ共有


# ChoreoFix 作業計画

最終更新: 2026-10-15。作業先はこのディレクトリ、リモートは
`git@github.com:hibanaworks/chorefix.git`。途中の実装を完成扱いしない。

## 目的と変更しない条件

元のTraceFixの48課題を縮めずにHibanaで実装し、論文化に必要な比較データを集める。
参加者、資源数、依存関係、独立して進められる仕事、障害条件を維持する。
比較側の実装や課題をHibanaに合わせて変更しない。

- 原論文の4条件とHibana: 48課題 × 3障害条件 × 3反復 × 5条件 = 2,160実行。
- TraceFix最新版の補足比較: 432実行。Hibana側432実行と対応付ける。
- 合計予定2,592実行。採点済み比較実行は現在0。
- 実験参加モデルはすべて `gpt-6-luna` / `xhigh`。
- Hibanaは公開済み `=0.9.9`。WASI runtimeの現在の固定revは
  `7905fa07bfd66db69ea7fb74883b7ca1e2b572bd`。
- 原論文側rev: `87b01a2c0ae53c5045c9ce3d43cd0235b1831275`。
  最新版側の記録済みrev: `36b4bd108fcd74f7efb17613aebbe3fdf55499a9`。
  最新版と呼ぶ比較を確定する前にリモートを再確認する。
- 開発サブエージェントは使わない。無関係な作業コピーや`hibana-pico/`は変更しない。
  本体の不具合を見つけたらバージョンと最小再現を報告する。本体を修正する場合は
  remote最新版を基準に隔離した場所で作業し、Lean・Z3・Miri・Rust testを通し、
  修正版を実際に使うhibana-quicへの影響も確認する。古いログで代用しない。

## 設計上の必須条件

**ヘルパー・ラッパーによる隠蔽なし。別のステートマシンなし。
タスク進行のフラグ管理なし。連携・権限・完了の二重管理なし。**

順序・許可・分岐・再試行・完了をヘルパー、ラッパー、driver、callback、
filesystem、transportへ移さない。段階enum、program counter、許可表、
approved/done、所有者記録などに名前を変えても禁止。
Endpointだけが手順の進行と許可を持ち、実際のEndpoint操作をlocalsideへ直接書く。
codec・buffer・オブジェクトの具体化は`imp/`へ置けるが、そこに手順の権限を持たせない。
完了報告前に、説明だけでなく実コードと検証結果でこの条件を確認する。

連携手順の仕様源はglobalだけ。`seq` / `send` / `route` / resolver /
`par` / `roll`を必要な箇所で使う。localsideにはEndpoint操作を書き下し、
どのメッセージが次のstepを消費するか追えるようにする。

第二の状態機械、段階enum、program counter、許可表、approved/doneフラグ、
二重完了管理、localsideを隠すヘルパーやラッパーを作らない。
Tokioのcurrent-thread async/awaitで実行する。

モデルはシェルで作業方法を選ぶ。固定コマンド列や限定コマンドを連携制御の代わりに
しない。普通のコマンド失敗や引数ミスはモデルに返し、修正して続けられるようにする。
共有操作の拒否・再考は明示的なHibanaの経路として扱う。
元課題で終端となるdomain failureを、勝手に再試行可能な課題に変更しない。

権限と順序はprojected Endpointが持つ。wasip1-runtimeは実際のWASI要求と
完了を扱う。ChoreoFSは受け入れた要求をオブジェクト操作として具体化する。
ChoreoFSのパス/fd/rightsの事実だけでタスク効果を許可しない。
共有操作のopen/write/closeをタスクglobalへ接続し、受信stepと必要な許可stepの
消費後に効果を出す。汎用JSON RPCの後段に別の手順検査を置く構成で済ませない。

## 現在の状態

- この作業を始めた時点のpushは `c49ab0aaa5b7cff685e7a237ec446e9ead71ddba`。
  そのコミットのGitHub CI成功は確認済み。
- 既存の実装は1E/M/H、2E/M/H、8E、9E/M/H、16E。
  今回の更新には6Eのコードと試験も含めた。
  全要件を受理済みの完全移植は現在0。
- 6E追加後の既存経路はRustのcheckとPython 23試験が通った。
  これを新しいChoreoFS接続の検証結果に転用しない。
- 6EのLunaシェル試行はdepth 0と2が各1件、depth 1が成功1件・失敗1件。
  失敗資料は `results/failures/review-feedback-depth1-001-6E.json`。
  失敗試行と中断したusageを捨てず、成功率の分母から除かない。
- **現在の実験用シェルはまだ限定コマンド方式で、汎用 `/tools/call` を使う。**
  シェルの自由度と正しいChoreoFS接続は未達。
  既存のシェル試行は旧接続の観測資料として残すが、目標の境界を検証した証拠にはしない。
- 16Eの直接接続を `src/tasks/cicd/choreofs/` に実装中。
  globalにサービスopen/write/close、資源の貸し借り、build→test→deploy、
  入力不正の拒否と再提出を記述し、localsideにもEndpoint操作を直接置いた。
  手動生成したWASI要求による機構試験3件が通った。
- タスクglobalと3組の一般I/O globalを`par`で合成し、同じ元から11rolesを投影した。
  `cicd-wasi`は3つの実ゲストを動かし、サービスの実WASI要求をタスクlocalsideへ渡す。
  `/tools/call`経由の別validatorへ戻す経路はない。
  `tests/test_cicd_wasi.py`の4試験が通った。障害設定0/1/2の各回で、
  私有ファイル、普通のコマンド失敗、不正なサービス名、不正引数からの再提出、
  先行するtest/deployの待機、元のsimulatorによる3操作、完了後の重複拒否を確認した。
  作業前のguest終了は失敗であり、domain効果も成功判定も出ないことを確認した。
  ログは`output/direct-wasi-pytest-004/`。固定コマンドによる境界試験であり、
  モデルの自由度や比較成功率を検証したものではない。
- fmtと`cargo +1.95.0 clippy --locked --all-targets -- -D warnings`は通った。
  Rust全28試験とPythonの自repo全27試験も通った。
  pytestの探索先は`tests/`へ固定し、2つの未変更upstreamや調査用cloneを
  自repoの試験と混在させない。upstream試験はそれぞれを明示して別途実行する。
  この新しい経路のLuna試行、汎用シェル、全48課題の受理はまだ完了していない。
  `ChoreoFS File`のバッファはデータであり、タスク状態や追加の権限には使わない。
- 自己送信の分岐を`branch.send`で消費できると思ったのは呼出側の取り違えだった。
  現行0.9.9では`branch.recv`が空のローカル操作を消費する。自己送受信なので
  これを通信方向の不整合とは断定できない。本体は変更していない。
  根拠は`results/validation/hibana099-local-action-api.md`。
- 汎用シェル候補を`guests/bash/`に実装した。rust-bashの固定revをWASI向けに
  パッチし、実ファイル操作はguestの`std::fs`を通る。別のFSやJS実行系を使わない。
  stable Rust 1.95のrelease buildとstrict Clippyは通ったが、まだ動作済みではない。
  実moduleは5,255,753 bytes、101 types、7,108関数、1,937 table entries、
  最大540 locals、最大body 83,955 bytes、初期memory 43 pages。
  25 importsはすべてWASI Preview 1で、非WASI importは0。
  現runtimeでの起動は`Wasm(Unsupported(10025))`で拒否された。
  このコードは`too many core wasm function types`に対応する。simulator効果は0。
  容量調査であり、モデル実験や採点済み実行には数えない。
  原ログは`output/bash-guest-probe-001/`、宣言値は`output/bash-guest-profile.json`。

## 次の一手

1. シェルの言語・作業用ファイル・利用できるプログラムを整え、モデルが自由に
   コマンドやスクリプトを組み立てられるようにする。実際のWASI runtimeの
   syscallとVM容量の制約も解決する。別エンジンやhost shellで代用しない。
   候補シェルはまず実WASI moduleの依存importと容量を測り、JS importや
   別のFSが必要な実装を動作済みとして採用しない。
   次はWASI runtimeのcaller-provided workspace化を進める。224関数・32locals等の
   固定配列を巨大化した別modeやheap fallbackを追加しない。関数bodyのu16位置も
   実moduleの83,955 bytesを表せる形へ直す。小さい組込みguestの予算を維持する。
   その後、seek/作成/削除/rename/link等の実WASI要求をglobalとChoreoFSへ追加し、
   作業用ファイルの固定8名制限と4095-byte入力制限をなくす。
   シェル内の変数・構文の状態と、禁止するタスク連携の二重管理を混同しない。
2. 実Luna xhigh参加者で再検証し、元のsimulatorと独立監査を通す。
   その後、残りの移植にも同じ境界を適用する。

## 比較・論文までの残りのゲート

- 全48課題の完全移植と独立したtask fidelity受理。
- 原論文側の全48課題の実行可能な公開資料の確認。現状の不足を補ったふりをしない。
- 元の4条件を変更せず実行できることと、最新版の補足比較の固定。
- ソース、バイナリ、guest、モデル設定、raw events、usage、失敗を凍結して採点。
- 2,592実行と独立監査。安全性、完了率、token、時間、再考、変更負担を分析。
- README、結果一覧、再現手順、論文PDFを実際の証拠に合わせて更新する。
  成功率や速度の優位性は比較が終わるまで主張しない。
- fmt / clippy / 関連試験 / 必要なcross-repo検証を通してpushし、GitHub CIを確認する。

未完了を完了に書き換えない。作業を進めたらこの「現在の状態」と「次の一手」を更新する。

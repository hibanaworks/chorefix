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
  Rust全28試験とPythonの自repo全29試験も通った。
  pytestの探索先は`tests/`へ固定し、2つの未変更upstreamや調査用cloneを
  自repoの試験と混在させない。upstream試験はそれぞれを明示して別途実行する。
  この新しい経路のLuna試行と全48課題の受理はまだ完了していない。
  `ChoreoFS File`のバッファはデータであり、タスク状態や追加の権限には使わない。
- 自己送信の分岐を`branch.send`で消費できると思ったのは呼出側の取り違えだった。
  現行0.9.9では`branch.recv`が空のローカル操作を消費する。自己送受信なので
  これを通信方向の不整合とは断定できない。本体は変更していない。
  根拠は`results/validation/hibana099-local-action-api.md`。
- `rust-bash`候補は採用しない。5,255,753-byte moduleの起動は容量超過で
  拒否されていた。失敗資料は`results/validation/bash-guest-qualification.json`に保存し、
  guest本体・依存パッチ・準備スクリプトは現行経路とCIから除いた。
- 独立リポジトリ`https://github.com/hibanaworks/hibana-wash`を作成した。
  初期コミットは`38a168e929d27df904bdcff3c1d647ae84af47af`。
  `vendor/hibana-wash` submoduleで固定し、ソースの別コピーを実装として持たない。
  dependency-free / no_std / no allocatorの小さいWASI shellである。
  引用符、変数、条件、反復、パイプ、リダイレクト、保存したscriptを実装し、
  タスク名・段階・許可表は持たない。guest内FSやnative FS経路も持たない。
  実体は6個のWASI importsを使い、既存のprojected EndpointとChoreoFSへ渡る。
  正確なmodule宣言と採用範囲は`results/validation/wash-qualification.json`に保存する。
  wash単体のRust test/build/strict ClippyとGitHub CIは通った。
- 16Eの実WASI接続はwashへ切り替えた。従来の4試験と、新しいscript機構試験1件が
  元のsimulatorで通った。複数行、変数、for/while、if、pipe、redirect、script読込、
  エラー後の続行、計算予算の終了と再考、私有ファイル名の選択を確認した。
  private objectは8個まで、名前はmodelが選ぶ。名前や内容はタスク権限を与えない。
  既存の制限guestは旧経路の回帰試験用に残す。旧model preflightはまだその経路であり、
  washのmodel試験として数えない。新経路へのharness移行とLuna実行は次の作業。
- `a1ece11608ae19c8f548bac52f15b531236f1f80`のCIで、タスク拒否後に
  別のmodelからRPCが届くと、閉じたログへの書き込みエラーで元の拒否を隠す競合が出た。
  実際の終了verdictを送信境界で保持し、RPC Futureの回収を修正した。
  意図的にlogger/stream終了後へ送信する回帰試験を加え、Python全28件が通った。
  これはRPCの寿命処理であり、タスク進行や許可をPythonへ移す変更ではない。
  修正版`7fa7406f8b8a0f2a5d3f2247496be796214694b6`のGitHub CI成功も確認した。
- runtimeの更新は既存コピーを触らず
  `output/upstream/hibana-wasip1-shell`の`development/caller-workspace-shell`で作業中。
  関数ごとにlocals型配列を複製せず、宣言をmodule bytesから借用する変更を加えた。
  localsの値は宣言型で初期化され、書き込み前に型を検査する。
  関数bodyの制御位置をu32へ広げ、80,000 bytesを飛び越すif/block/loop試験も加えた。
  現在のRust 80件（2 ignored）、strict Clippy、thumbv8m no_stdは通った。
  VM 26,512 bytes / CodeBody 32 bytes / Frame 560 bytes。
  u32変更前の関連Miri 5件とruntime gatesは通ったが、u32変更後には再実行していない。
  この変更は未push・実験未採用で保留する。washには既存の固定revを使う。

## 次の一手

1. `hibana-wash`の残る資源境界とscriptの組合せを検査し、原論文の課題を実行する
   model harnessを16Eの新しい実WASI・ChoreoFS経路へ移す。
   特定タスクのコマンド列を組み込まず、modelがscriptを組み立てる。
   現在の64KiBと公開WASI runtimeで進め、大型Bash用のruntime拡張は再開しない。
   shell内の構文・変数・exit statusと、禁止するタスク連携の二重管理を混同しない。
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

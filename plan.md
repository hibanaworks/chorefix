# ChoreoFix 実験計画

更新: 2026-10-30。目的は、全48課題でHibanaの連携・所有権の強制を評価し、
比較実験と論文に必要なデータを集めることである。採点対象は **0 / 2,592実行**。
予備試験、部品試験、ビルド成功を採点済み実験へ数えない。

## 固定する比較条件

- 原論文4条件とHibana: 48課題 × 3障害条件 × 3反復 × 5条件 = 2,160実行。
- 最新TraceFix監視条件: 同じ48課題・障害条件・反復で432実行を追加する。
- まず最新版432件と対応するHibana432件を優先し、全2,592件を完了する。
- 原論文比較は `87b01a2c0ae53c5045c9ce3d43cd0235b1831275`、最新版比較と入力は
  `36b4bd108fcd74f7efb17613aebbe3fdf55499a9`。固定前にremoteの変更を確認し、
  既存の記録済み版を黙って置き換えない。比較側のsource・実行エンジンは変更しない。
- 全条件の参加モデルは `gpt-6-luna`、`xhigh`。既存ChatGPT契約のApp Server接続を使う。
  APIキー課金、別モデル、effortの代替を許さない。開発サブエージェントは使わない。
- 課題の目標、参加者、物理資源、安全性、必要なデータ依存と元simulatorの採点を保つ。
  ChoreoFixの手順は改善してよい。変更は採点前に
  [実験契約](benchmark/experiment-contract.json)へ記録する。
- seed、遅延、timeout、tool上限、実行数は[実験設定](experiment.json)と生成matrixで固定する。
  新たな生成比較protocolは、モデル検査だけでなく元のdomain toolとの一致も確認する。
  不適合な派生入力は再生成し、baselineの課題失敗として数えない。

## 実装上の契約

Globalを唯一の連携仕様にし、localsideへEndpointの操作を直接書く。
順序・分岐・再考・並行処理・完了をhelper、wrapper、flag、所有者表、別の状態機械へ移さない。
資源の実体を保存するbufferと、資源の操作権・進行を区別する。

- Global: `seq`、`send`、`route`、resolver、`par`、`roll`。
- Localside: `send`、`recv`、`offer`、`par`を直接記述し、独立した操作をTokioで並行駆動する。
- 共有操作に必要な実Endpointを所有値で渡す。そのEndpointで実引数を送り、
  実結果を受けて同じEndpointを返す。空のgrantと別経路の操作権を併存させない。
- 復旧可能な失敗は、実結果をresolverへ渡し、routeとrollで再考へつなぐ。
  元課題が終端失敗を指定する場合は、その結果を保つ。難しい課題を単発試行へ縮めない。
- モデルはコマンド、私有計算、解法を自由に選ぶ。各モデルに独立した実Wasm＋hibana-washを
  一つ割り当てる。各2MiBのcaller-supplied memoryを使う。
- すべてのFS操作は、対応するWASI要求をEndpointで受信した後にChoreoFSへ渡す。
  私有bufferでもraw要求による迂回を作らない。ChoreoFSの表示と操作権を混同しない。
- アプリ資源、ファイル内容、連携状態を`Rc`、`Arc`、`RefCell`、`Mutex`で共有しない。
  データは参加者が所有する。resolver入力は実結果であり、別のworkflow cursorにしない。
- 外部RPCの相関と記録はI/O担当の責務であり、taskの許可を与えない。
  既存の共有や旧経路は整理対象である。別名への置換で完了としない。

## 現在のsourceと検証範囲

ChoreoFixのmanifest/lockは本体
`3e2220a85368e03b8b0fa099c39c7ec13557a200`、WASI
`4df26995ae1affef6fd2d559cfeeac2c05833b5f`を選択している。
通常のGit依存を使い、診断用のpath patchを認定へ流用しない。

| 対象 | 現状 | 残る確認・変更 |
| --- | --- | --- |
| 1E/1M/1H | 実資源Endpointの要求・結果・返却。1Hは7人が実結果で再考。実Endpoint・実Wasm試験と実モデルの予備記録あり | source固定後の全条件の認定と比較 |
| 2E/2M/2H | 両sectionの実Endpoint要求・結果・返却。実Wasm15条件と3/5/7人のLuna予備試験が通過 | source固定後の全条件の認定と比較 |
| 6E | globalとlocalsideを実Repo Endpointの貸出・要求・結果・返却から全面刷新。再考、途中のレビュー対象変更、並行処理を投影で駆動 | 全crate build、Clippyと実Wasm12条件が通過。元scoreはすべて完了・資源違反0件。ローカルcore修正を選んだ検証であり、通常Git pin、資源監査と実モデルの認定は未完 |
| 8E | DB・test環境の実Endpoint委譲、単独所有の結果ファイル、実Wasm3条件、Luna成功・終端失敗の予備試験 | source固定後の認定と比較 |
| 9E/9M/9H | 実fork Endpointを両方受け取り、実要求・結果・返却。3/5/7人の実Wasm・実モデル予備試験あり | source固定後の認定と比較 |
| 16E | build/deploy環境の実Endpoint委譲。実Wasm3条件、早期終了拒否、3人のLuna予備試験あり | source固定後の認定と比較 |
| 3E | nativeの研究report protocolと意味試験あり | 実Endpoint委譲・実Wasmへの移行 |
| その他の課題 | 元の48入力・825 checklist項目を照合するmanifestとmatrixあり | 各課題のglobal/localside、実Wasm、比較入力の認定 |

同じ本体pinを使っていても、別sourceの予備記録を最終source全体の認定へ読み替えない。
固定script、実モデル、元simulatorの完了、protocol完了を別々に報告する。

直近の16E変更では全target build、native56件、Clippy、fmtとsource前後hash一致が成功した。
全子孫の最大memoryは1,418,739,712bytes。実Wasmの元障害条件0/1/2はすべて元score成功・
違反0件、実資源要求・返却各3件だった。各14変異を拒否し、作業前のguest終了も効果0件で拒否した。
シェルの私有ファイル、パイプ、ループ、構文エラー後の修正、51,200Bの出力と実memory growthも検査した。
これらの試験はRust runnerへ移し、対応する自作Python試験は削除した。

3人のLuna xhighによる16E予備試験も13.14秒で完走し、inclusive 131,503tokens、
全員のChatGPT認証・モデル・effort一致、元score成功・違反0件を確認した。
組み込みと別processの同じauditorが一致し、実モデル記録の14変異も拒否した。
これは監査アルゴリズムが二つ独立にあるという意味ではない。

2E/2M/2Hも空の資源通知を除去し、実Endpointの並行受け取り・実要求・実結果・並行返却へ移した。
全target build、native56件、Clippyが成功し、build前後のsource hashは一致した。
元の障害3条件、終端レビュー失敗、通常遅延を各人数で実Wasm実行し、15条件が通過した。
各14変異、合計210件を拒否し、終端失敗を元scoreの失敗として保った。通常遅延では
3/5/7人でそれぞれ1/2/3件の独立したwriteが重なった。旧writing監査CLIとPython監査は削除した。
raw記録、実行binary、Wasm、sourceとhashを保持し、再生成可能な変異入力のコピーはhashと結果を残して削除した。
3/5/7人のLuna xhigh予備試験も完走した。全員のChatGPT認証・モデル・effort・usageを確認し、
元score成功、違反0件、実資源要求・返却は各6/10/14件だった。実モデルで観測した同時writeは
それぞれ1/1/3件である。inclusive tokensは328,617 / 526,721 / 743,788、所要時間は
21.58 / 20.18 / 21.14秒。比較性能の結果へ読み替えない。model試験全子孫の最大memoryは2,178,547,712bytes。
18記録すべてについて別processで同じauditorを実行し、組み込み監査と一致した。
3つのmodel記録も各14変異を拒否した。両監査は実行processが別であり、アルゴリズムは同じである。

6Eはglobalとlocalsideを全面的に書き直した。ファイル操作の反復はWASIのprivate I/O
choreographyに置き、業務のglobalには実要求、唯一のRepo Endpointの貸出、実操作、
実結果、返却を記述する。開発者の仲介roleとレビュー用補助viewは削除した。
最初のレビューと次にレビューする対象は実際の要求から選ぶ。再考中に他方のレビューへ
切り替えられ、独立した実装・レビューとmerge・レビューは実際に並行駆動する。
mergeと正常終了は承認された枝へ置く。選択と実レビュー結果は、分岐を知る必要がある
参加者へglobalの通信で伝える。進行はEndpointが管理し、アプリにphaseや完了flagを置かない。

全crateのbuildとClippyは成功した。buildは255.68秒、全子孫の最大memoryは
1,212,514,304bytesだった。実Wasmでは、元障害深さ0/1/2、最初のレビュー対象A/B、
一方を承認まで続ける順序と交互にレビューする順序の12条件が完走した。
元scoreはすべて完了・資源違反0件であり、深さ2の交互順序では両featureが実際に
一度ずつ失敗してから成功した。早いmerge、誤った引数、完了後の重複要求は効果を起こさない。
この検証はAIを呼ばないRustのshell刺激であり、ローカルcore修正を選択している。
通常Git pin、実資源の記録監査、並行した効果の観測と実モデル試験の認定は残っている。

1Hの委譲版には、通常のmodule localsideを動かさず、現在のglobalを公開`project`で
直接駆動する排他試験を追加した。A/Bそれぞれのdesign・implementationを完了し、
AがDB用Endpointを保持する間はBの受信がPendingとなり、Aの実要求・結果・返却後に
同じEndpointをBが取得できる。保持中の別attachも両者について拒否される。
この1つの操作列はローカルcore修正を選ぶmodule単位の試験で通過した。
全スケジュールの証明や通常Git pinでの全crate認定としては数えない。

旧native RPC、専用CLIと小型shellは削除済み。実験用transportは一つの値がqueueとbindingを
所有し、runtimeが借用する。現行の外部JSON接続は`ExternalIo`がstdin/stdoutと応答対応を所有する。
API結果ファイルはファイル担当が内容・offsetを所有し、metadataもglobalの要求・応答で処理する。
残る旧native codebaseの要求queue、試験用journal、runnerの記録・callbackの共有は整理する。

## 本体・統合の保証

本体は`feature/endpoint-delegation`で開発し、mainの変更・リリースはレビュー後に扱う。
coreのRust 2024、no_std、heap割当なし、caller storage、lockなしを維持する。
採用Pico/Pico 2のstack/SRAM/flash予算、命令数・コンパイル容量gateを保つ。
`long_running_const_eval`のallow/expectは禁止し、元のglobalとforbidで検査する。

現在の本体は生成に`OperationIndex`を再利用し、actor行の検査は生成iteratorから独立している。
検査済みactor bindingを信頼する範囲と、source所有権との別の照合を区別する。
関連Rust変異試験、Lean/Z3、strict Miri、Kaniと、100role/4,000eventのARM投影build、
1,542eventの実Endpoint試験が成功している。後続`b38178a`は試験bufferのリーク修正であり、
consumerの実行コードは変わらない。

本体の全体CIは確認中であり、現在版の全Kani・全Miri成功とは扱わない。
旧pinのMiri生成木群は180秒で時間切れになっている。停止を成功へ変えず、必要な検査を通す。
アプリ/executor/transportの前提とモデル定理、Rust実装の試験を区別し、任意Rust全体の完全証明を
追加の無期限な条件へしない。具体的な不一致が見つかったら、その修正を試験拡大より優先する。

生成検査は、深さ1全数、同型を除いた小さい合法形、長さ4〜5の入れ子相互作用、深いseed付き
抽出と縮小を区別する。両側が複合的なpar、選択前の受信、委譲先のchoice/roll/par/再委譲を含める。
合法形は実Endpointを並行駆動し、初手だけでなく継続も確認する。非合法形はコンパイル拒否を確認する。
決めた対象表の完了を網羅済みと呼び、全構文・全scheduleの証明と混同しない。

TLS/QUICは修正coreを実際に選択した全体試験・CI・44件interopで統合を確認する。
ACK、再送、TLS、輻輳制御、timer、byte offset等の必要データは保ち、引継ぎflagや二重完了表を除去する。
本体be87736を使うQUIC `7fd3fad`のCIと44件interopは成功済みだが、現在pinの認定へ流用しない。
CI待ちの間も独立した実装作業は続ける。

## 残る工程

1. **全48課題を移行する。** 既存6系の操作権を実Endpoint委譲へ移し、残る各familyを実装する。
   元のデータ依存、安全性、再考、独立した仕事の並行性を検査する。比較側は変更しない。
2. **自作実行基盤をRustへ統一する。** 旧nativeの進行管理、未使用入口、自作Python runner/監査を
   置き換えて削除する。変更しない上流PythonはRust PyO3から利用する。
3. **比較モデル接続を認定する。** 五つの未変更比較clientと元generatorについて、実際のChatGPT接続で
   prompt、可視history、schema、tool順序、圧縮を保持する。16要求のlocal HTTP fixture成功だけでは足りない。
   隠れた会話状態で、上流が削ったhistoryを復活させない。同じhistory方針をHibanaにも適用する。
4. **最終sourceとartifactを固定する。** source/lock/binary/Wasm/model/configのhashとbuild receiptを保存する。
   snapshotの古い`.cargo/config.toml`必須条件、auditのmodel呼出し数の扱いも修正し、凍結経路を実試験する。
   各task/depthの正常・拒否・障害経路、資源の唯一性・結果の忠実性、並行性、auditorの変異感度を確認する。
5. **2,592実行を行う。** matrixの各identityを一度ずつ実行し、全条件のraw model events、shell/domain events、
   元score、usage、source/binaryの対応を保存する。harness失敗、再実行、欠測を別に記録する。
6. **集計と論文を完成させる。** 安全性、完了率、inclusive tokens、待機を含む時間、再考、並行性、
   実装・手順変更負担を比較する。reasoning tokensをoutputへ二重加算しない。
   対応する反復を使って不確実性と負例を報告し、結果に合わせて論文・README・再現手順を整える。

論文は実験データを優先してから仕上げる。TraceFixへ対抗する主張は、実測と保証の範囲で述べる。
予備試験の成功を成功率・速度・品質の優位性へ外挿しない。

## ローカル作業と完了判定

重い検証は同時に一つ、Cargo/Kani jobsは1。全子孫合計3,000,000,000bytesで停止するguardを使う。
現在は余裕を取り2,750,000,000bytesのguardで実行する。上限停止は未検証として残し、
予算の引上げ、globalの縮小、検査の無効化で成功へ変えない。検証前後のsource hashを照合する。
最新stableで開発し、Miri/Kaniだけは各ツールが必要な検証compilerを使う。

既存のdirty sourceを保ち、同名packageの別checkoutでtarget directoryを共有しない。
本体等の検証済み修正は開発branchへpushし、研究repo `hibana-tracefix`はローカルで作業する。
古いbuild・不要cache・未使用の実験成果物は削除し、profilerと論文に必要なraw証拠は保持する。

完了には、全48課題の実装・認定、未変更比較側の接続認定、固定した全2,592実行、独立採点・集計、
論文と再現手順、必要なcore/TLS/QUIC/WASIの最終gateが必要である。未実行・間接的な証拠は完了に数えない。

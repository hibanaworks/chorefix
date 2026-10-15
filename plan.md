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
- まず最新版TraceFix 432件と対応するHibana 432件、計864件を優先する。
  旧4条件は追加比較として残す。現時点で全体予定から削除したとは扱わない。
- 新しいツール、実験runner、監査、Hibana統合はRustを基本にする。
  未変更のTraceFix比較実装と元simulatorに必要なPythonは保持する。
- 実験参加モデルはすべて `gpt-6-luna` / `xhigh`。
- Hibanaは公開済み `=0.9.9`。WASI runtimeの現在の固定revは
  `4ebdad1443cf49488eae924afe528f67f5a5953b`。
- 原論文側rev: `87b01a2c0ae53c5045c9ce3d43cd0235b1831275`。
  最新版側の記録済みrev: `36b4bd108fcd74f7efb17613aebbe3fdf55499a9`。
  最新版と呼ぶ比較を確定する前にリモートを再確認する。
- 開発サブエージェントは使わない。無関係な作業コピーや`hibana-pico/`は変更しない。
  本体の不具合を見つけたらバージョンと最小再現を報告する。本体を修正する場合は
  remote最新版を基準に隔離した場所で作業し、Lean・Z3・Miri・Rust testを通し、
  修正版を実際に使うhibana-quicへの影響も確認する。古いログで代用しない。
  本体の不具合が確定したら必ず本体を直し、再現回帰試験を残す。
  アプリ側の回避処理で本体修正を省略したり、修正完了と報告してはならない。

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

## 接続修正前の記録

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
- **従来のmodel preflightは限定コマンド方式で、汎用 `/tools/call` を使う。**
  16Eには後述のstd版wash・直接ChoreoFS接続のLuna試行を追加したが、
  残りの課題はまだ同じ境界へ移行していない。
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
  初期版はno_stdだったが、現在は可読性を優先したdependency-free / stdのWASI shellである。
  引用符、変数、条件、反復、パイプ、リダイレクト、保存したscriptを実装し、
  タスク名・段階・許可表は持たない。guest内FSやnative FS経路も持たない。
  現在の実体は13個のWASI importsを使い、projected EndpointとChoreoFSへ渡る。
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
## std版washとruntimeの更新

- washはstdのString/Vec/BTreeMapとstd::fs/std::ioを使う。
  guest内FSやnative FS経路は持たない。共有redirectのcloseエラーを保持し、
  directoryだけは既存WASI境界の96-byte replyに合わせて要求する。
- std版washは`12e137aba2cea74f353c198a8d7c92afc977627c`をpush済み。
  単体parser 3件、strict Clippy、GitHub CIは成功。
  moduleは136,726 bytes、349関数、23 types、124 table entries、最大19 locals。
  WASI以外のimportsは0。ChoreoFixのsubmoduleもこのrevを使う。
- runtimeは隔離コピー`output/upstream/hibana-wasip1-shell`で修正した。
  `hibana = "=0.9.9"`は既存の依存を維持し、metadataでも確認した。
  locals宣言をmodule bytesから借用し、active frameだけへ展開する。
  localsはpayloadと唯一の型tagへ分け、書き込み前に型を検査する。
  関数bodyの制御位置をu32へ広げ、80,000 bytesを飛び越すif/block/loopも検証した。
- 関数384、table128、call frames64、value stack128、control frames512、
  control targets320。VM実測51,472 bytes / CodeBody24 bytes / Frame336 bytes。
  VM予算64KiB、default guest込み128KiB、Pico2 fixtureのRAM予算160KiB。
  Pico2 link実測はflash79,848 bytes、24KiB stack予約込みRAM136,664 bytes。
  実機動作やworst-case stack測定は、このlink結果には含まれない。
- runtime full gatesは成功。native Rust 83件、Miri 81件（2 ignored）、strict Clippy、
  Lean 30定理と実VM/ChoreoFS 680判断の照合、Z3 4 UNSAT/3 SAT、
  Pico2、WASI std demo、doc、packageを通した。
  ログは`results/validation/wasi-std-runtime-gates.log`。
  runtimeは`4ebdad1443cf49488eae924afe528f67f5a5953b`を
  remote defaultの`development/affine-wasi-boundary`へpush済みで、GitHub CIも成功。
  古い32KiB予算の成功を新しい結果として使わない。
- ChoreoFixでは各guestに1MiBのbackingを用意し、最初はmoduleの64KiB。
  `MemoryGrowReq`をlocalsideで受信し、`would_fit()`の結果を`MemoryGrowRet`で返す。
  メモリ許可も既存globalを通す。stdlibのheapと実行器のstack容量は区別する。
- 隔離試験ではstd moduleの実接続5件と51,200-byte pipeのheap拡張が通った。
  ChoreoFixでも公開runtime revを選び、実接続6件を含むPython全30件、
  Rust全28件、parser3件、fmt、全targetと両guestのstrict Clippy、
  inventoryと比較matrix不変を確認した。
  採点済み比較は引き続き0。runtimeと採用側8406c11のGitHub CIは成功。
- `tools/run_direct_wasi_preflight.py`で16Eのモデルharnessを直接接続へ移した。
  元のdescription・schemas・static step dataを渡し、作業コマンドはモデルが選ぶ。
  task permissionやprogressのPython側コピーは持たない。
  archiveのupstream revは親Gitへ問い合わせず、検証済みsource manifestから読む。
- Luna xhighの直接接続4試行を`results/direct-wasi-preflight.json`へ保存した。
  001は完了したが初期記録器のupstream rev誤記があり、最終採用試行にはしない。
  002はroot directoryとpreopenに同じfdを割り当てた接続ミスで失敗した。
  003はprotocol/simulatorが完了したが、exit時のstdout回収漏れと終了後の呼出で
  model1人がharness_errorになった。どちらも失敗資料とusageを残した。
- ChoreoFS factsでpreopen fd3とopened root fd15を分けた。終了時には実際の
  terminal receiverを消費して出力を返す。終了後のcallはclosed I/Oの応答となり、
  他の参加者を止めない。タスクの進行フラグや第二の許可表は加えていない。
  回帰試験はroot listing後の私有作業、共有効果、exit直前の出力、終了後のcallを含む。
- 修正後004は3人ともcompleted、元simulatorの3操作が各1回成功し、違反なし。
  32.151秒、inclusive total_tokens=256,564。全source snapshot、binary、raw events、
  model/effort/usage、効果順序を保存後に別scriptで監査した。
  誤model、誤rev、改変usage、欠けたdomain eventも監査が拒否することを確認した。
  auditは`results/audits/direct-wasi-wash-std-004-16E.json`。
  4試行の全資料は`output/direct-wasi-preflight/wash-std-20261015-evidence.tar.gz`。
  16Eは元から全depthで障害注入を無効にしている。再考や比較の成功率には数えない。

## 接続設計の修正

ユーザーの指摘により、従来の`par(task, private_io)`と`mpsc`での接続を未達と判定した。
WASI側Endpointを出た後に別のRustチャネルでタスクroleへ要求を渡していたため、
shared effect直前のEndpoint検査だけではglobalに接続全体を書いたことにならない。
既存の4回のLuna試行、機構試験、CIは部品の動作資料として保管する。
raw-event auditの成功はこの設計条件の受理を意味しない。

最小の修正はWASIを駆動するrole自身をサービス要求の送信元にすることである。
`VM Endpoint -> global request -> service owner Endpoint -> ChoreoFS -> global reply`
を同じglobalに記述する。`mpsc`/oneshotによるgraph間転送は削除する。
WASIの値とmessageの変換はlocalsideで明示し、進行フラグや第二の許可表は作らない。
まず16Eでこの経路と元要件を検証してから6Eへ進む。

## 現在の検証済み状態

- 16Eのサービス転送用`Import` enum、mpsc/oneshot、task-client relayを削除した。
  WASI側自身がglobalのOpen/Write/Commitを送り、所有者が受信する。
  `src/tasks/cicd/choreofs/localside/wasi.rs`に実際のEndpoint操作を書き下した。
  ChoreoFSのcodec・オブジェクトbytesだけを`src/shell/imp/service.rs`へ置く。
- 先行して届くOpenと前段の完了通知は`par`で独立に受ける。
  資源の貸借、共有効果、build→test→deployは`seq`で結ぶ。
  引数の検証結果をresolverが読み、拒否からの再提出を`route`と`roll`で表す。
- 成功後のサービス拒否と後続の仕事を`par`に置いた。
  元課題のdeployment-complete通知2件も残し、各所有者は受信まで完了しない。
  サービス終了は明示的な分岐。後続待ちを代用するflagやRustチャネルはない。
- Endpoint機構試験1件と実WASI試験6件が通った。
  不正引数からの再提出、先行要求、chunked write、後続を待たない重複拒否、
  原simulatorの3効果、root listing、私有script、heap拡張、終了を確認した。
  作業前のexitは投影された送信stepに合わず失敗し、domain効果を出さない。
- 実Transportの送受信・requeueを追記専用ログへ残す。
  ログは権限や進行判断に使わず、各効果より前のWASI→所有者close受信と、
  実結果の返送、元課題の4通知を別scriptで検査する。
  close欠落と送信元改変を検査が拒否することも確認した。
- 16Eのprivate I/Oは自分の/workを随時読み書きし、自分の/service/resultを随時読む。
  読み取りもEndpointとChoreoFSを通るが、共有readのtask依存制約は未検証。
  そうした制約を持つ元課題では、read要求・前提・応答もglobalに書く。
  private namespaceの分離とresultのread-onlyは実ゲスト試験で検査する。
- 現在のRust全26試験、Python全30試験が成功。
  元48課題825 checklistと比較matrixの内容は不変。
  本体・WASI runtime・washのソースは今回変更していない。
- この接続でLuna xhigh 3人の試行`projected-wasi-005`が完了。
  41.353秒、inclusive total_tokens=253,654。3効果、4通知、実request/replyを
  保存ログから別scriptで監査した。監査は
  `results/audits/projected-wasi-005-16E.json`。
  ソース・binary・raw eventsのarchiveは結果台帳にhash付きで記録した。
  e89db351の最新CI（38699645780）は成功。16E全要件の独立したfidelity受理は未完了。
  全48課題の受理は0、採点済み比較も0のまま。旧4試行は部品資料として保管する。


## 6Eの直接WASI接続と反復境界の調査

- `src/tasks/review/choreofs/`、`review-wasi`、実ゲスト試験を追加中。未commit・未push。
  この作業の全要件受理はまだ0。採点済み比較も0のまま。
- 共有REPOのopen/write/closeを実WASI Endpointからglobalで運び、同じglobalの
  REPO所有者がChoreoFSを具体化する。A/Bの並行作業とレビュー差し戻しもglobalに置いた。
  連携mpsc、所有者表、段階enum、進行フラグ、Endpointラッパーは加えていない。
- 最初は`offer`を通常の並行受信のdispatchとして誤用しPhaseInvariantになった。
  次に全要求をrouteで表し、REPOの単一offerループで要求とresolver分岐を扱った。
  ネストしたofferが必ず直前のcloseの判断を返すという仮定も除いた。
- 現在の公開0.9.9では、反復内と後続が同じOpen/Write契約を使うと、前の反復を
  選ぶ失敗が残る。4役だけのunit-payload例でも再現し、remote main
  `f1c4dd4bab211b005687cfec15bb53a1f77500ed`でも同じ結果を確認した。
- 既存Lean GlobalSemanticsは意図した後続と古い反復への再入の両方を受理する。
  従って「反復を単純に閉じる本体バグ修正が正しい」とは断定しない。
  globalの通信契約とAPIが選ぶ経路の曖昧さを解消する必要がある。
- 隔離コピーの2候補は最小例を通したが実6Eのdepth 0/1/2を通せなかった。
  未消費event優先の候補はRust workspace 868 passed / 12 ignoredでも、実6Eは
  ResolverReject 264で3件失敗した。候補は両方とも取り消し、本体へ採用していない。
- 本体の元作業コピー、WASI runtime、washは変更していない。本体の隔離コピーには
  失敗を保持する回帰試験だけを残した。Lean/Z3/Miri/QUICの全ゲートは未実施。
- 証拠と解釈は `results/validation/roll-boundary/README.md` と `result.json`。
  実ゲスト失敗、ソース、候補patchのarchive hashも記録した。
  最小例や通常Rust試験の成功を6E完成の証拠にしない。


## 反復の投影と寿命の追加調査（未採用）

- 全ての後続送信とroll先頭を比較する隔離候補は、Lean全ゲート・Z3・Miriの
  実validator試験4件・Kaniの実validator harness 3件を通した。
  修正候補を実際に選ぶ現行QUICもdefault 569件、hq 574件が成功した。
  ただし、この候補は従来動いていた16Eのglobalを拒否するため採用しない。
  workspace全体もCI-tool fixture試験の5失敗が残り、全面成功ではない。
  これらは次の寿命候補に引き継げる検証結果ではない。
- 別の隔離コピー`output/upstream/hibana-scope-lifetime`で、後続操作のcommitと
  以前のrollの関係を調べた。複数writeを含む4役再現と実6E depth 0は通るが、
  depth 1/2は`developer_a offer`のPhaseInvariantで失敗した。未採用。
  同laneの後続完了を走査するコードは診断用であり、最終実装にはしない。
- 独立した投影反例も保存した。globalの先頭は0→1の異なるmessageだが、
  role 2からは消え、反復先頭と後続先頭が同じ2→3 messageになる。
  現行の投影検査はこれを受理した。拒否を要求する試験1件は意図どおり失敗、
  異なるlocal先頭後のreply再利用と独立par内rollの肯定例2件は成功した。
  この検査のproductionコードはremote base f1c4dd4から変更していない。
- `results/validation/roll-scope-20261015/`に候補patch、再現source、ログ、hashを保存。
  本体・QUICへの採用やpushはしていない。採点済み実験と全要件受理は0のまま。

## 以前の作業候補（下の最新チェックポイントを優先）

1. 反復の投影条件と局所的な寿命を一緒に定義する。roleに無関係なglobal先頭で
   識別したことにせず、投影後の操作契約で曖昧さを調べる。
   `par`内rollの独立性、区別できる後続後のmessage再利用、外側rollによる
   内側visitの再開を回帰条件として保つ。既存の受理だけを正しさの根拠にしない。
   最終kernelは投影済みの正確な順序・所有権の事実を使い、走査による推測や
   send候補の優先順位で直さない。API aliasや利用側の進行状態も増やさない。
2. 意図した意味にLeanモデルとcheckerを合わせ、最小再現、Rust repo cfgを含む試験、
   Lean・Z3・Miri・必要なKani、修正を実際に選ぶ現行QUICを通す。
   実6E depth 0/1/2の差し戻し後の再提出、承認前merge拒否、REPO排他、
   レビュー中の独立した開発、全通知とguest終了が通るまで本体修正を採用しない。
3. 元課題のfidelityを独立に監査してからLuna xhighの実試行へ進む。
   その後、残りの移植にも同じ境界を適用する。std版washと公開runtime、
   各guest 1MiB backing、Tokio current-thread async/awaitを維持する。

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

## 本体修正の方針確定

ユーザーの指摘どおり、globalから一意に決まるなら投影が情報を保持し、同じ公開操作で
一意に決まらないならコンパイル時に拒否する。runtimeの送信優先順位では直さない。

- 16Eの既存実行成功を根拠に、強化した検査を過剰とした判断は撤回した。
  現行のGlobalSemanticsは、par合流と後続の受信後にも古いrollへの再入を許す。
  この正確な縮小例で、新しい後続sendとroll再入後の旧sendを両方受理することを
  Leanで確認した。同じrole・label・schemaから二つを選び分けることはできない。
- したがって今回の修正は反復の寿命の変更ではなく、現行の反復の意味に対する
  投影検査の不足を直す。`fix/roll-reentry-projection`を隔離コピーに作成した。
  roleごとの反復先頭と、routeで排他的ではない後続の送信契約を検査し、
  競合は既存のReentrySelectorで拒否する。kernelと公開APIは変更しない。
- 追加した診断9件、const projectionのcompile-fail、独立par内roll、通常のseq、
  受信を先頭とする反復の返信再利用、排他的routeの肯定例を回帰条件にした。
  Rust workspaceはrepo cfg付き・test-threads=1で878成功、失敗0、明示ignore 12。
  並行実行で失敗したCI-toolのfixture試験は、変更していないbaselineでも失敗し、
  単一threadではbaseline・修正版とも成功した。今回その試験のsourceは変更しない。
- Miriは実投影validatorとbranch ancestryの対象4件が成功。Kaniは本番validatorを呼ぶ
  4harnessが成功した。symbolicなu8 label/u32 schemaを縮めず、固定4eventまでの
  fixtureにunwind 8を使い、unwinding assertionも検査した。
  本体のstable no_stdとthumbv6m build、workspace Clippyも成功。
- Lean/Z3の新しい10命題と正確なUNSAT/UNSAT/SATは成功。Lean全ゲートと修正版を
  実際に選ぶ現行QUICのdefault/hqを再検証中。この結果を確認して修正を確定する。
- 寿命を走査から推測する旧候補は採用しない。ChoreoFixの16E/6Eも新検査に適合する
  choreographyへ整理する必要がある。既存の16E試行を全要件受理には数えない。
  全要件受理0・採点済み0は変わらない。

## 本体修正の公開チェックポイント

- 本体修正を`f86451b3fa65508e351fd48db32af5129a0539b6`としてcommitし、
  `fix/roll-reentry-projection`へpushした。PRは https://github.com/hibanaworks/hibana/pull/8 。
- Lean全ゲートも再実行して成功。修正版を直接依存とTLS経由の両方で選ぶ現行QUICは、
  default 593件、hq 598件（各24 doctest込み）が成功し、失敗0。
- Rust 878件、Clippy、no_std、thumbv6m、Lean、Z3、対象Miri 4件、
  本番validatorのKani 4harnessを確認済み。検証sourceとログのhashは
  `results/validation/roll-projection-fix-20261015/status.json`に保存した。
- mainはまだf1c4dd4。GitHub CIはpush run 38717977624とPR run 38718027357で実行中。
  registryの0.9.9は変更せず、新版の公開も行っていない。
- 次はこの本体の投影条件に合う直接WASI choreographyへ16E/6Eを整理する。
  APIやtask進行フラグで経路を足さない。元の課題要件の独立監査が終わるまで
  全要件受理0・採点済み0を保つ。

## 修正版本体の外部QUIC interop

- 593成功・598成功は通常構成とhq構成の別々の試験数で、どちらも失敗0。
  これらは外部peerとのinteropではなく、QUIC内の試験とdoctest。
- 修正版本体`f86451b`を直接依存とTLS経由の両方で選ぶ検証ブランチを
  QUICへpushした。QUIC commitは`2115eed459e6bc98b6293cbf4d4adabc8daea7f6`。
  draft PR: https://github.com/hibanaworks/hibana-quic/pull/1 。
- 外部44ケースは https://github.com/hibanaworks/hibana-quic/actions/runs/38720774555
  で実行中。Linuxのplan jobは成功。結果が揃うまでinterop合格とは数えない。
  失敗時の診断用鍵はrepo/CI外に保存し、CMS AES-256-GCMの復号往復を確認済み。
- 全ての独立consumerのCargo lockとpatchが同じcore commitか検査する。
  HQ driver check、source pinning 4試験、matrix verifier 13試験、
  staged QUIC/TLS source auditは成功。mainとregistryは変更していない。
- 別途進めている16Eの直接WASI接続には、ownerの明示的な反復開始と
  owner→WASIのRequestReadyをglobalに記述した。修正版本体を使う隔離consumerで
  Rustの対象1試験と実guestの4試験が成功。完全移植受理や採点済み実行には
  まだ数えない。6Eの整理とfidelity監査は未完了。

## 直接WASI反復の最新チェックポイント

- 16EのglobalはownerのServeRequest/RejectRequestと、owner→WASIの
  RequestReadyを明示した。各共有open/write/closeと終了でその受信を消費する。
  反復の選択や段階を表すアプリ状態は追加せず、Endpoint操作をlocalsideへ直接置いた。
- 公開済み0.9.9でRust 30試験・Clippy・実guestの4試験が成功した。
  新しい6E直接WASI試験を除いた既存Python 30試験も成功。除外した6Eを
  成功扱いしたり、全48課題の検証へ広げたりしない。
- 同じ16E sourceは修正版本体f86451bを使う隔離consumerでも、対象Rust 1試験と
  実guest 4試験が成功。mainの依存はregistryの=0.9.9のまま。
- Luna xhighの新しい非採点試行readiness-20261015-001は3参加者とも完了し、
  元simulatorの3操作が成功した。独立した保存イベント監査も成功。
  11 shell calls、384 frames、220,215 total tokens、27.127秒。
  課題記述・metadata・tools schema・16E simulator・CICD基底simulatorは
  原論文revと最新版revで同一bytesであることを確認した。
  source snapshot、raw model/carrier events、usage、hash、監査結果を
  results/validation/direct-wasi-readiness-20261015-001/ に保存した。
- これは比較2592実行の代替ではない。全要件受理0・採点済み0を保つ。
- 6Eは未確定。最初のreadiness候補はdepth0成功・depth1/2失敗で、
  2回目の実装後のWASI role0 recv ResultがPhaseInvariantとなった。
  別候補はdepth0がマージで停止し、depth1/2はREPO offerがPhaseInvariantとなった。
  どちらも採用済みとしない。output/review-readiness-fixed-* と
  output/review-aliased-outer-head/ にsourceとログを保持した。
  2役/3役/4役への縮小では再現しないものもあり、本体とglobalのどちらの問題か
  まだ確定していない。推測でkernelへ優先順位やアプリ進行状態を追加しない。
- 本体PR8のGitHub CIはpush/PRの両方でKaniとfinal-formが全成功した。
  QUICの最初の外部interopは42件成功、ECN 2件は解析ツールの起動失敗で未実行。
  同じ2115eed/f86451bに固定したECNだけの再試験を
  https://github.com/hibanaworks/hibana-quic/actions/runs/38723148382 で実行中。

### 次の一手

1. ECN再試験の実結果と、全caseのcore/source/pinを成果物で確認する。
   最初のworkflow自体はqualification失敗なので、緑と記録しない。
2. 6Eの停止を、投影した同じglobalと実際のEndpoint操作に沿って縮小する。
   globalが公開操作から一意でないなら投影で拒否し、一意なのに失敗するなら
   本体の実装を修正する。既存のpar内rollと有効な外側反復を維持する。
3. 6Eのdepth0/1/2、共有REPO排他、差し戻し後の再考、独立開発、全guest終了を
   通し、WASI→Endpoint→ChoreoFS→元simulatorの保存イベントを監査する。
4. その境界を残り48課題へ広げ、原論文4条件と最新版を変更せず比較し、
   全2592実行・独立監査・分析・論文を完了する。未完了を縮めて完了とはしない。

- 16Eの実装とLuna試行資料は25a2290としてmainへpush済み。
  外部QUICの42件はjob名だけでなく、ダウンロードした各方向verdict、
  core/TLS pin、source tree hash、simulator、ケース集合、終了コードから再確認した。
  成果物はinterop-attempt1-artifacts.tar.gz、集計はinterop-attempt1-audit.json。
- 6Eの失敗した2候補のsource、実carrier events、stderr、試験ログと、
  成功する縮小例をresults/validation/review-wasi-failures-20261015/へ保存した。
  これらの候補を本体の修正済み・完全移植済みという証拠にしない。

- ECNだけの再試験38723148382も成功した。成果物を再検査し、2方向の実verdict、
  source pairの全体hash、Hibana/TLS revision、共通pinsが初回と一致した。
  修正後の外部interopは同じsourceで全44件成功・候補失敗0・未実行0。
  初回workflow失敗と再試験のqualification skipも隠さず記録し、
  単一workflowでの全matrix合格とは区別する。資料はinterop-combined-audit.json。
- 25a2290のChoreoFix GitHub CIは成功した。16Eの変更と新しい非採点試行資料を含む。
- 6Eの診断は別clone output/upstream/hibana-recv-diagnostics で進めている。
  公開予定の本体PR8/f86451bとQUIC検証branchはcleanのまま。エラー出力を加えた
  診断用コピーで、最初の候補のResult失敗はkernel/recv/contract.rsの
  live_recv_lanesで有効な候補が一つもないところだと確認した。framed receiveの
  unique-matchより前に失敗している。現在はそのrole0/label24のroute scope選択を
  記録する診断buildを実行中。診断出力の追加を本体の修正とは扱わない。

- 6Eの診断buildと再試験も完了した。role0がResult(label24)を要求する時点で
  候補event8/18/28が全てdisabledで、review scope12の選択1が観測される。
  完全frameのunique-matchやpayload処理に入る前のlocal contract検査で失敗する。
  この観測だけで本体不具合・global非投影可能・localsideミスのどれかとは断定しない。
  診断patch・compiler flag・source base・raw stderr/イベントを保存した。

## 2026-10-15: 6Eの受信エラーと並行resolver停止を分離

- f86451bの3役native再現で、非選択枝のpar依存が外側roll再開を妨げる本体不具合を特定。
  依存の条件は先行領域に適用されるもので、完了済みの別枝のeventを消す条件ではない。
  1e55aabで修正、40b5448で回帰試験のメッセージ名を訂正して同じPRへpushした。
  本体Rust 879成功・0失敗・12明示ignore。追加Lean 9主張とZ3 3組SAT/UNSAT成功。
  最小再現のMiriはLinux対象で成功。Mac対象の初回MiriはTokioのkqueue未対応で停止。
  no_std、thumbv6m、Clippy成功。40b5448のGitHub CIは確認中。
- 新しい本体を選択したQUIC検証ブランチは99fcbf7。interop run 38727610451は進行中。
  ECN jobは失敗しており、原因と成果物は未監査。他の成功を全44成功へ読み替えない。
  先のf86451bでの44件成功と新しい本体の検証は別に保存する。
- 6Eの最初のreadiness候補を隔離consumerで再試験するとdepth0/1成功、depth2停止。
  ただしdepth2単独では成功し、再試験ではdepth1も初回の処理で停止した。
  差し戻し2回そのものが原因とは言えず、並行処理順に依存する本体の別不具合がある。
- WASI/ChoreoFSを外した3役のglobalでも、2要求を先に受信すると、片方のresolver付き
  ローカル処理を残してofferがPendingになる最小再現を得た。診断コピーだけで調査中。
  readinessを広げるだけの候補は古いreview判断を再び選ぶため採用しない。
  実行可能な投影済みstepと、過去の反復のroute選択を区別して根治する。
- 失敗と候補の境界はresults/validation/parallel-resolver-20261015/に保存。
  アプリ側のフラグ、手順ラッパー、完了の二重管理は追加しない。比較の採点実行は0のまま。

## 2026-10-15: 本体の並行resolver修正と静的送信分岐の回帰

- 並行resolver停止は本体の不具合だった。5440370で、投影済みの有効なlocal/send入口を
  持つresolverを、新しい受信を待たずに選べるよう修正した。実行可能性は枝の選択権限ではない。
  resolver/frameが引き続き選択を所有し、進行中の枝は変更できない。
- 5440370のQUICローカル試験で2件の実回帰が出た。完了した反復の選択を見分ける変更が、
  静的sendで確定したcontrollerの選択まで無効として扱い、続くofferをPhaseInvariantにした。
  2役のseq(route(send,send), route(reply,route(reply,reply))).roll()でも再現した。
  受信入口の除外だけを疑った初期仮説は再試験で否定した。
- c2001ec06944d7f3b299e16b5b3efb5b0ba0366aで、送信で確定したcontroller選択と、
  完了したobserver/resolver反復を分けた。静的送信選択は応答を探す間も保持する。
  過去のobserver選択は実行可能な並行resolverを止めない。新APIやアプリの進行フラグは不要。
  回帰試験とMiri gate、GlobalSemanticsの3主張、Z3の3組SAT/UNSATを追加してPR8へpush済み。
- 現在の本体Rustは883成功・0失敗・12明示ignore。Clippy/fmt/no_std/thumbv6m、
  対象2再現のMiriと関連4 Kani harnessが成功した。追加Leanは10主張、Z3は7SAT/6UNSAT成功。
  full Leanの再監査、GitHub CI、本体を実選択したQUIC全体/外部interopは確認中。
- 6Eのreadiness候補は同じ最終本体でdepth0/1/2を5巡、計15試験成功。差し戻し2回だけで
  起きる問題ではなく、並行実行順に依存する不具合だった。これはモデルなしの境界試験であり、
  採点済み2592実行・全課題受理の代わりにしない。全要件受理0・採点済み0を維持する。
- 5440370の外部QUIC run38731137608は失敗。candidate endpointの実行エラーが含まれる。
  古いsourceの成功やインフラ失敗と混ぜない。修正後commitを全consumerに固定して再検証する。

- c2001ecのfull Lean gateも成功した。709 static theorem、506 generated theorem、
  458 generated obligationとcompiled claim typesを再監査済み。Git pinを実際に選択した
  6Eの追加3条件も成功し、元simulatorでの独立replayと共有資源区間/通信同一性監査が成功。
  結果を1件除いたログコピーは拒否された。証拠はstatic-controller-continuation-20261015。
  ChoreoFix全体Python33件も成功した。これを採点比較や全要件受理には数えない。

- c2001ecのclean checkoutでQUIC default593成功・hq598成功、各0失敗。
  QUIC 44d291aでは全6 consumerとlockを同じGit c2001ecへ更新し、直接とTLS経由の
  一致をcargo treeで確認。Git pinのingress3件、source pin4件、matrix13件、driver試験が成功。
  新しい外部interopはrun38735739939。結果は未確定で、本体CIと合わせて成果物を確認する。
  ChoreoFixは9877301で6E境界と証拠をmainへpush済み。Rust26/Python33/Clippy/fmt/
  原課題inventoryが成功。本体mainのmerge/registry公開と採点比較は未実施。

## 2026-10-15: 6Eの対象bindingと実モデル経路

- 実モデル試行前の検査で、6EのA向けサービスへBの引数を渡しても実行できる不足を発見。
  これは本体の順序検査の問題ではなく、サービスとdomain対象の対応を検証していなかった。
  wrong-target-before-001で、誤ったreviewが元simulatorまで届くことを保存した。
- globalに2つのfeature/merge branchのdomain identityを明記し、service ownerがclose時に
  そのidentityとの一致を検査する。実際の検証結果を既存resolverに渡し、既存routeの
  invalid_argumentsへ返す。別の権限表や手順フラグは追加しない。正しい値で再提出可能。
  比較側や元simulatorのfuzzy matchingは変更しない。このbindingによる入力制約は
  Hibana側インターフェースの条件として実験に記録し、API差がないとは主張しない。
- A先行/B先行の両方で差し戻し0/1/2を試す6条件が成功。モデルにレビュー順を指定せず、
  拒否された対象が実効果にならないことも確認した。全要件受理と採点済み比較は引き続き0。
- run_direct_wasi_preflight.pyに--task 6Eを追加。原課題description、元schemas、元の
  static step dataを渡し、自由なshellと操作対象のobjectだけを説明する。
  command recipe、Pythonの許可管理、シミュレーター状態からの手順判断は追加しない。
- 共通モデル監査は要求/実応答のLuna xhigh、raw thread/turn記録、shellの送受信、
  inclusive token数を検査する。欠落/置換/失敗turn/重複RPC/不正集計/別promptを拒否する。
  保存済みの実Luna16E試行は新監査でも同じ集計となった。新6E監査は元simulatorの
  replay、共有資源区間、実close→effectの経路、実レビュー結果がdeveloperへ戻った
  時系列と内容、正しい対象の両mergeを検査する。
- 全体検証の初回はローカル容量不足で失敗。診断用と作業用の再生成可能なincremental
  cacheを整理し、ソース/実験ログ/実行binaryは残した。約6GiBの空きを確保した。
  再試験はCARGO_INCREMENTAL=0、CARGO_BUILD_JOBS=2。環境失敗をコード成功へ読み替えない。

## 2026-10-15: 実モデル3条件の保存と独立サービスのEndpoint

- 直前のturnは進捗あり。不要な生成物10か所を削除して約17GBを解放し、ソース、
  全実験ログ、固定runtime/Wasmを保持した。現在の作業用targetも保持している。
- 4a9937f/c2001ecの実6E Luna xhighはdepth0/1/2で各1回完了。全9参加者、
  合計1,171,689 inclusive tokens。depth2はA/Bを交互にレビューし、各featureを1回
  差し戻した。先行A/B固定の決定的試験とは別の経路。採点比較や成功率推定にはしない。
  results/review-direct-wasi-preflight.jsonとreview-direct-wasi-20261015にsource、
  prompt、raw model/carrier、結果、独立監査を保存した。binaryを含む全archiveはoutputに残る。
- 監査は元simulatorのreplay、REPO区間、実close、code-ready、実review feedbackと
  再提出、両merge、全frame消費、実model/effort/turnとinclusive usageを検査する。
  7種類の改変コピーは外側hashを再計算しても拒否された。原資料は変更しない。
- 追加した完了後重複要求の試験は旧構成で6条件すべて失敗した。完了済みAの待受通知を、
  Bのclose応答を待つ同じEndpointのofferが観測した。offerをfd/request単位の待機と
  仮定したアプリ側の不足であり、この観測を本体の不具合とは扱わない。
- 最小の修正として、同じtask globalにREVIEW_A_WASI/REVIEW_B_WASIを定義した。
  元のreviewer guest自身が2つのservice Endpointとprivate I/O Endpointを直接駆動する。
  モデル/guestは元の3人のまま。別graph、転送channel、relay、進行flag、応答許可表、
  localsideを隠すwrapperは追加していない。未消費previewを別の状態として保存もしない。
- 修正後は重複review/implement/mergeと分割JSON書き込みを含む6境界条件、独立replay、
  Rust26/Python43/Clippy/fmt/inventoryが成功。修正前の6失敗とsourceも保存する。
  新しい構成の実Luna確認はこれから1回行い、旧3試行とsourceを混同しない。
- 本体c2001ecのCI38734656111はfinal-form/Kaniとも成功。QUIC44d291aの
  外部38735739939も全44cellが同一runで成功。個別phase/verdict、expected matrix、
  全groupのcore/TLS pinとsource treeを別途照合済み。reference self-test省略と
  runner/capture終了patchを保存する。本体main merge/registry公開は未実施。
- 論文は旧generic-RPCや別graphの部品試験を現行接続の証拠にしないよう改訂した。
  interface差、信頼する具体化role、未達の比較/保守測定を明記。LaTeX compileは成功。
  全48課題の全要件受理0、採点済み0、合計予定2,592を維持する。

### 次に進める範囲

まずサービス分離後のLuna6Eを監査・保存する。その後はwriting 2E/M/Hを優先し、
既存の別session resource brokerを同じglobalの実WASI要求/資源/効果/応答へ置き換える。
2Eだけの直列化を2M/Hへ一般化せず、5/7人の独立したwriteが並行できる条件を保つ。
reviewは元課題どおり終端で、余分なretryを加えない。続いて残る全familyを完了し、
比較側を変更せず48課題のfidelity、baseline corpus、採点実行と保守測定へ進む。


## 2026-10-15: 実験優先への指示とwritingの単一global化

- ユーザーの指示により論文の推敲は止め、残り実装と実験を優先する。現稿の質は未達。
- c07fe41のサービス分離後Luna6E depth2も3参加者で完了。396,886 inclusive tokens、
  22 shell/10 domain/1,153 frame、60.887秒。両feature各1回差し戻し、両merge成功。
  実caller11/12、4組の通知、元simulator replay、全通信、usage監査と7改変拒否が成功。
  source/生ログ/結果をreview-service-endpoints-luna-20261015へ保存。旧sourceと混ぜない。
- 次の実装としてwriting 2E/M/Hの別session resource brokerを除去中。
  全研究者と物理sectionを1つのglobal/sessionへ投影する。独立した準備をpar、
  disjoint writeをpar、各sectionの2貸借をglobalの順序へ置く。reviewは終端のまま。
  localsideには直接のsend/recvを残し、所有者表、段階enum、進行flag、bridgeは加えない。
  3/5/7人と資源数は維持する。最初のcompileは型名Dとrole定数Dの衝突で失敗し、
  型パラメータをWorkへ改名して検証中。これを受理済み移植とは数えない。
  native検証後は実WASI/ChoreoFS要求をこのglobalへ接続し、元simulatorと独立性を検査する。

- writingの投影拒否はreceive laneの送信元切替に因果handoffがなかったため。
  物理sectionから次の研究者へのAvailableメッセージをglobalに追加し、実localsideが
  recv後にAcquireする形へ修正中。ループ状態や所有者表は足さない。本体変更は不要。
  同時に2Mの2 write/2Hの3 writeが同時にdomainへ入るbarrier試験を追加した。

- writingは2資源の取得をparで分離し、実Available/Acquire/Grantedと返却通信を
  localsideへ書き下して投影が成功した。nativeの5試験が成功。2Mの2 write、2Hの
  3 writeが同時にdomainへ入ることをbarrierで確認し、reviewが資源を保持しない試験も成功。
  取得をseqにした中間候補は異なる送信元の受信を同じlaneに置くため投影で拒否された。
  診断を回避したのではなく、独立した取得をparで正しく記述した。本体は変更していない。
- c07fe41のChoreoFix CI38741479487は成功済み。新しいwritingの全体ゲートは実施中。

- writing単一globalの全体検証はRust27/Python43/Clippy/fmt成功。保存した元simulatorの
  実プロセス3試験のreplayも成功し、domain callsは12/20/28、実write並行数は1/2/3、
  section競合0。不完全区間と偽造結果のhash再計算コピーも監査が拒否した。
  証拠はwriting-single-global-20261015。次は同じglobalへの実WASI/ChoreoFS接続。
  ここはnative側の土台であり、全要件受理0/採点0を維持する。論文は編集再開しない。

## 2026-10-15: writingの実WASI接続とRust検証runner

- 必要な比較と追加試験を全て行う指示を維持し、最新版との864件を先に進める。
  旧4条件を削除したとは扱わない。新規のrunnerと監査はRustを基本とする。
- writing 2E/M/HのWASI caller、実物化owner、物理section、private I/Oを同じglobalへ
  接続中。順序や所有者を別のflag/enum/tableで管理せず、両側にEndpoint操作を明記する。
- 要求型を4つのrollで再利用した候補は再入先が曖昧で、実行用buildの投影が拒否した。
  cargo checkの成功を投影完了と誤認した点は訂正。段階ごとに要求/応答型を区別し、
  WASI localsideも4段階と完了後の受付を直接書く。汎用driverに段階selectorは持たせない。
- 資源の空き通知と個人の準備/要求をparに置く。実Acquireは有効なwrite close後。
  5人の2組/7人の3組の独立writeを維持し、Return後にreviewへ進む。
- writing-boundaryはRustの外部刺激runnerと保存ログ監査。変更しない元Python simulatorへ
  接続する最小のRPC部分だけoriginal_writing_simulator.pyを使う。
  不正引数、対象違い、早い要求、完了後重複、分割JSON、private計算、実closeから効果まで、
  資源競合と独立write、review中の返却を実guestで検査する予定。実モデル試験ではない。
- 7人分のglobalはrustcの既定const評価作業量を超える。stableのlint設定だけで有限評価を
  継続し、projectionの意味検査は省略しない。本体/API/元simulatorは変更していない。
  実行用buildと新runnerの検証は進行中で、受理/採点件数はまだ増やさない。
- 直前ab9172fのChoreoFix CI38744181443は成功。現在の未commit変更の成功とは混同しない。

- ユーザー訂正: 独立した資源取得のglobalはparを保つ。チェックリストの逐次取得表現を
  理由にglobalまでseqへ強化しかけた変更は取り消した。両grant→effect→returnsの依存だけを
  強制する。続く訂正でlocalsideも両Acquireを先に送信し、その後で両grantを待つ形へ変更。
  parを宣言しただけで実行を直列化しない。実際の通信で両要求が待ち合わせより前に出た
  ことを検査する。別endpointのclone、進行flag、owner表、wrapperは追加しない。
- 元実験の制御手順をコピーすることは目的ではない。成果物・参加者・資源独占・必要な
  データ依存・障害条件を保ち、Hibanaで自然な並行/分岐/再考を実装して比較する。
  逐次取得など元checklistの手順との違いを採点前に記録し、比較側と原資料は変更しない。

- 実guestの初回は検証runnerが未対応のshell算術構文を送って失敗した。標準で用意した
  exprへ刺激を修正し、失敗ログは2E-001に保存。次は同じresolver IDを全ownerへ登録した
  アプリ側の誤りでResolverRejectとなった。登録keyはglobal image+IDでありrole別ではない。
  2E-002と変更前sourceを保存し、各ownerのCHECK_A..Gへ分離中。本体は変更していない。
  native/実WASIのlocalsideは両Acquireを発行後にgrant待機する形へ変更した。
  Rust監査では両要求が最初のgrant受信より前に出たことを直接検査し、直列化ログの
  改変も拒否する。実行用buildの初回成功は5分28秒。現修正後の再build/試験は進行中。

- 最終版の実WASI 2E/M/Hは全員3/5/7人、原simulator12/20/28操作で完了し、共有資源競合0。
  独立writeの同時実行は1/2/3。最初のgrantが送られる前に両Acquireが出たことと、最初の
  返却ackが送られる前に両Returnが出たことをログで確認する。単なるpar宣言ではない。
  7改変×3構成=21改変が拒否された。直列化したコピーも全frameを保ったまま拒否する。
- 2Eの終端review失敗を元simulatorの既存FailureSpecで設定する別境界試験も成功。
  protocolは正常終了する一方、domainのis_completeはfalse。4操作/人のままでretryなし。
  本比較には混ぜない。失敗を成功へ読み替えず、protocolと成果物の完了を分けて保存する。
- Rust27/Python43/Clippy/fmt、元入力とbaseline runtimeのbyte inventoryが成功。
  最終Rust test buildは5分39秒。有限const評価の作業量lintだけを許可し、意味検査は保持。
  final source、実binary/Wasm hash、4試験のraw log、元simulator記録、21改変拒否、
  開発中の失敗をwriting-direct-wasi-20261015に保存。完全archiveはoutputに保持する。
  source archiveは作業treeのコードを固定したもの。全課題受理0、採点比較0/2592は変えない。
  次はこの接続のLuna試験と残るfamily、baseline corpus、全課題の条件分類を進める。

## 2026-10-16: Rustの実モデルrunnerと実並行性

- 新規の実験runnerと独立監査をtools/runnerへRustで実装。root libを依存にせず、
  実験用の変更のたびに7人globalのconst投影を再コンパイルしない。taskのEndpoint操作は
  既存のglobal/localsideに置いたまま。host側はmodel/stdio RPCの相関と観測だけを扱う。
- Tokio current-threadで全Luna参加者、実guestのstdout、元simulatorの独立effectを
  同時にpollする。modelの待機が別guestやdomain effectの読取りを止めない。
  全実行をgpt-6-luna/xhighへ固定し、実応答とnative tool禁止を検査する。
- 元simulatorの必須Python adapterだけにモデル用設定を追加。seed/depth/delayは
  experiment.jsonに合わせ、並行性確認用barrierは実モデル試験には入れない。
  元TraceFix、baseline、元inputは変更しない。決定的試験は既存barrierを維持する。
- Rustの3接続試験が成功。誤ったmodelとnative toolを拒否し、実wireのtool応答と
  inclusive usageを確認。別に実WASI/元simulatorの5人試験を明示実行して成功。
  人工barrierなし、元delay倍率1.0でwriteの実並行数2。これは固定入力、model_calls=0。
- 新監査は保存source/実binary、要求/実返答modelとeffort、raw turn重複、usage、
  実guestと同一のcommand/response、全モデルturnの重複区間、元simulator/frame監査を
  検査する。改変コピーの外側hashも再計算して拒否を確認する機能を用意した。
  これから実Luna 2E/M/Hを実行し、独立監査と改変試験を通して保存する。
- 4096a2aのCI38753129971はRust検証を18分33秒で通過後、WASI build中に20分上限で
  cancelされた。annotationが明示する実行時間超過であり、本体の失敗とは扱わない。
  検査を削らず45分へ変更し、Rust runnerのfmt/test/Clippyと明示integrationを追加。
  Python43件と旧2H/終端失敗ログの再監査も成功。採点済み0/2592、全要件受理0を維持。

- 実モデルrunnerの初回001は、モデル開始前のsource inventoryで停止した。
  元manifestは48task配下を対象とするが、新監査がenvironments直下のpackage fileも
  比較したため。元inventoryと同じ範囲へ修正し、全48task/simulatorのbyte一致を
  Rust回帰試験に追加する。停止ログを残し、別run IDで再開する。Hibana本体の問題ではない。

## 2026-10-16: writingの実Luna 3/5/7人の並行動作

- execution source9ed7738、固定runtime4096a2a/c2001ecで2E/M/Hを各depth2で1回実行。
  actual Luna xhighは計15人。全participantのturnが3/5/7人で重なり、自然なdomain writeは
  同時1/2/3件。元delay倍率1.0、人工barrierなし。原simulator60操作完了、共有資源違反0。
  shell91回、inclusive tokens1,674,159。時間は30.419/32.886/37.335秒。
  どの実行も採点比較へは数えない。実測値はresults/writing-direct-wasi-preflight.json。
- 原実行のfrozen protocol verifierは、同じsection pairのカンマ空白や並び順まで固定して
  正しい要求を拒否した。serviceはpair集合で検証し、元simulatorもこの表記を受け入れる。
  監査だけを修正し、2M固定刺激も逆順/空白なしの同一pairへ変更して回帰試験にした。
  wrong target、余分な引数、欠落/重複、実効果、競合の検査は残す。本体/consumer実行は
  変更していない。原実行のbinary/source/rawは書換えず、新verifierを明示して再監査。
- 新Rust監査が3条件とも成功。model/usage/command/reply/score/sourceの改変を各6件、
  計18件拒否。外側hashも改変へ合わせた上で拒否する。2M実guestの新表記回帰と
  7種のprotocol改変拒否も成功。直列取得を偽装したログは引き続き拒否する。
- 001はinventory範囲の誤り、002はClippyでbuild停止後に旧binaryを起動したため、
  ともにモデル開始前に停止。失敗と空ディレクトリ/ログを保ち、成功の003へ混ぜない。
  raw model/carrier、prompt、usage、source manifest、execution source、監査sourceと
  verifier hashを保存する。完全なbinary付きarchiveはoutputに残す。
- obsolete CI8a31fc8は新しい9ed7738が開始したため明示cancel。旧4096a2aの20分timeoutと
  区別する。9ed7738の45分CIはまだ実行中。新しい監査修正のCI結果は未確定。
  次は残るfamilyの実WASI接続、48課題のfidelity/手順差の分類、原baseline corpusと
  採点比較へ進む。全要件受理0、採点0/2592、論文推敲停止の指示を維持する。

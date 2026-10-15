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

## 次の一手

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

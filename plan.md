# ChoreoFix 研究開発計画

更新: 2026-10-31。受領した「ChoreoFix 研究開発計画」計画版1.0を反映した。
入力文書のSHA-256は[実験契約](benchmark/experiment-contract.json)に記録する。
研究の中心は、**実資源の唯一の操作権、残存手順、実際の共有効果を結ぶ保証**である。
委譲、投影、並行処理、prefix safety、版の照合をそれぞれ単独の新規性とはしない。

既存A比較の採点対象は **0 / 2,592実行**。追加B–Gは **0 / 1,368実行**で、
最大600実行の予備枠を含む総上限は **4,560実行**である。
追加課題、比較接続、精度設計は未認定であり、この上限を凍結・実行済みmatrixとは扱わない。
予備試験、部品試験、ビルド成功を採点済み実験へ数えない。

## 研究質問と範囲

| 問い | 必要な証拠 |
| --- | --- |
| RQ1 権限と効果 | 一意性、継続保存、受理済み要求と実効果の対応 |
| RQ2 合法な自由度 | 実結果による再考、異なる合法手順、独立作業の実際の重なり |
| RQ3 業務の完了 | 元の業務採点、不正効果、拒否、protocol完了を別々に測る対応比較 |
| RQ4 実装と保守 | 同じ変更に必要な宣言、localside、権限制御、テストの修正負担 |

第一論文は既知の有限role集合、実行時の委譲先選択、複数資源、結果依存分岐、
有限に実行された再考と独立作業の並行性を扱う。安全性は有限prefixにも適用する。
任意の動的トポロジー、永続クラッシュ復旧、外部APIのexactly-once、任意Rustの完全証明、
自然言語の意図の完全理解は完了条件に追加しない。範囲外の課題はnative未対応と記録し、
簡略化して成功にせず、通常のrun失敗にも変換しない。

## 最初の技術判断点: T3/T4と実効果

主ケースは、Aが資源Rで候補v1を作り、独立したCが資源Sで作業し、失敗後にAが
同じEndpointをBへ渡し、Bが候補v2の検証と実結果の受信を経てcommitする手順とする。
旧A、v1の承認、別の引数、片方しかない資源で実効果を起こせないことを確認する。
競合が通常のbroker、lease、proxyや再計画で同じ業務を完了しても成功と数える。

この[最小機構ケース](src/tasks/handoff)を実装した。AとBは別runtimeで、
失敗した候補からBが再修正・再検証し、RとSの両要求がそろった後だけ版付きの値を公開する。
独立したCはSで計算し、その実結果も公開物に含める。進行はglobalのpar、roll、routeと
直接のEndpoint操作で行い、資源の現在版・検証結果・公開物は物理サービスが所有する。
これはnativeのmemory-backed fixtureであり、実Wasm、実モデル、実成果物Gや採点実験ではない。

通常の再考、旧承認・別対象・引数変更・負の検証結果・左右の資源要求の不一致6種類、
片方の要求だけを送ったPendingからの継続、未修正候補への承認偽装、実際の否定結果による
修正なしの失敗終了、計10ケースが通過した。
引数違反では公開0件と初期bindingの再発行拒否、正常系では同じEndpointのA→B→Aと
正しい版の公開を確認した。通常Git pinの個別fixtureでは5試験に含まれる10ケースとClippyが成功した。
再修正／候補確定はglobalの明示的routeであり、localsideのofferはその選択を観測する。
同じGit pinの10ケースをstrict provenanceのMiriでも確認した。Miriの実行は240.71秒、
その後のLean照合を含む検証process全体の最大memoryは705,691,648bytesだった。
検証用の時間枠は計装コストを含む60秒で、実験のdeadlineやmemory上限とは別である。
このケースの追加で本体の挙動は変更していない。

[Lean](proofs/README.md)では、任意のtarget・版・値について、両要求と実検証結果・現在版が
一致することと公開判定の同値性を証明した。既存coreのsource実行から操作点の重複がないことも
導いた。`EffectReceipts.lean`では、同じsource実行上で各効果が両資源の実受信を消費する
receiver契約と合成し、両要求・実検証結果・現在版の一致と、どちらの受信も二度目の公開へ
流用できないことを証明した。異なる資源の受信は別の出現になる。各要求の送信roleを対応する
資源へ結び、両資源のcustody履歴が同じsource wordへ射影される条件で、両方の権限上限と
効果の受信対応を合成した。許可する公理はpropext、Quot.soundのみ。
このreceiver契約はadapterの信頼境界であり、任意Rustから自動的に導かれるとはしない。
各受信・引数・実効果を投影と実行記録に結ぶ認定、残る受信側と他の実引数の対応は未完である。
新しいruntime台帳や出現IDは追加していない。

| 目標 | 命題と完了の証拠 | 現状 |
| --- | --- | --- |
| T1 完全実行の対応 | 評価で使うmessage/seq/par/route/resolver/roll/委譲について、globalと理想投影の完全実行を双方向に対応させる | 既存core定理の適用範囲と必要補題を対応付ける。研究全体の証明完了とはしない |
| T2 有限prefixの対応 | 非同期seq、join、分岐知識、任意の有限roll展開、移譲後の継続が合法prefixを保つ | 既存出現モデルを再利用する。深さ0/1/2の試験を一般命題の代わりにしない |
| T3 一意性と継続 | 初期binding、送信future、移送中、受信値、失効を含む全遷移で利用可能権限が高々一つで、継続が保存される | coreのcustody関連定理、1H排他試験、同じsource word上のreceiver契約との最小合成あり。具体的なR/S両binding・実受信・効果入口への適用と最終sourceでの認定は未完 |
| T4 実要求と実効果 | 各共有効果は一つの受理済み動的操作に対応し、対象・版・引数が一致し、必要な全資源と先行結果を満たす。拒否は効果も業務commitも生まない | 両資源の受信を消費する最小receiver契約下で、データ照合と受信の再利用禁止を合成済み。実要求・結果のlocalsideと記録監査あり。具体的な全効果入口との対応、他の引数への一般化と最終sourceでの認定は未完 |
| T5 実結果による分岐 | resolverが同じ動的操作の実結果で分岐し、別対象・旧版・以前のrollの結果を流用できない | 結果駆動の再考試験あり。対象・版・承認の一般的な対応は未認定 |
| T6 進行と条件付き完了 | 到達状態を正常完了、明示的失敗、外部応答待ち、合法遷移可能に分類し、終了する部分のprefixを既決定のまま延長できる | 既存進行定理の前提と適用範囲を照合する。非協力的guestの終了を保証しない |

T3の所在はghost stateとして定義し、runtimeへowner表や新しい権限tokenを追加しない。
利用可能な所有値と、受理後に送信futureが保持する権限ゼロの証拠を区別する。
送信側の局所I/Oエラーを、受信側へ移った権限の失効や再発行の根拠にしない。
初期発行と物理aliasの唯一性、FIFO・非重複配送・保持者からの受信可能性は明示した境界に置く。

`Domain::execute`は通常のhostメソッドである。受信した要求、実引数、必要資源の対応を
確認する箇所を[効果境界とadapter契約](benchmark/effect-boundaries.md)へ列挙する。
現在の一覧はWasm移行済み12課題の6familyを対象とし、残る36課題の認定とは区別する。
WASI import、公開host API、codec、物理bindingからの
迂回・引数差替え・再送を実効果の直前で試す。任意host Rustの効果を型だけで禁止したとは言わない。
新しい許可wrapperでEndpointを置き換えない。

最初の負例は旧所有者の遅延要求、二重移転、引数差替え、旧結果・旧承認、必要資源の欠落である。
故障は貸出、受理、結果受信、返却など同じ論理checkpointへ入れ、拒否直後の実資源を確認する。
現在の記録変異試験は監査の感度の証拠であり、これらの実行時防止試験とは別に数える。
効果成功後に応答が失われたら結果不明とし、権限を再発行せずfail-closedで終了する。

意味論のLean定理、投影データの検査、Rustの差分試験・Miri・必要なKani、Wasm/codec/adapter、
独立採点の信頼範囲を分ける。独自の未証明の正しさaxiomや未完証明を完成として公開しない。
アプリの適合性、公平なpoll、応答、容量、再考の終了は条件付き完了の前提として記録する。
`mem::forget`で権限やfutureを放棄するアプリまで終了させるとは主張しない。

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

### 比較版

| 対象 | 計画の基準SHA | 認定 |
| --- | --- | --- |
| ChoreoFix | `0954383fe40c5a6e14d330b13c709ff9ff0db9fd` | 更新計画が参照する版。後続の公開進捗やdirty sourceを本比較版と同一視しない |
| TraceFix 原論文 | `87b01a2c0ae53c5045c9ce3d43cd0235b1831275` | 接続・入力認定は未完 |
| TraceFix 最新 | `36b4bd108fcd74f7efb17613aebbe3fdf55499a9` | 接続・入力認定は未完 |
| ZipperGen | `349e0c05b150139a1e1b25243eaf0f9edcb3baad` | 新規。source、native表現範囲、接続、保証の監査が必要 |
| Concordat | `0e3b185167b963df41c68b6dbacd3488c1063e3d` | 新規。source、native表現範囲、接続、保証の監査が必要 |

これらは受領計画の基準版であり、全接続の実験済み認定ではない。変更時は理由と再認定範囲を残す。
TraceFix最新版の順序検査、ZipperGen最新版のparallel/spawn/CPL、Concordatの人間決定・予算分割を
無効化した比較を各方式の代表にしない。文献上の主張は固定sourceと一次資料で確認する。
関連研究は受領計画の[POPL 2008](https://mrg.cs.ox.ac.uk/publications/multiparty-asynchronous-session-types/)、
[ZipperGen論文](https://arxiv.org/html/2604.17612v3)、[CPL論文](https://arxiv.org/html/2605.20923v2)、
[Concordat論文](https://arxiv.org/html/2610.37461v1)を含めて位置付ける。

### 実行予算と課題選定

| 区分 | 設計 | 新規実行数 |
| --- | --- | ---: |
| A1 | 最新版TraceFix432 + ChoreoFix432 | 864 |
| A2 | 原論文4条件を各432 | 1,728 |
| B 共通課題 | 12課題 × 3障害 × 3反復 × ZipperGen/Concordat | 216 |
| C 航空業務 | 50課題 × 通常/pressure × 3反復 × Concordat/ChoreoFix | 600 |
| D ZipperGen例 | 4workflow × 2結果 × 3反復 × 2方式 | 48 |
| E 引継ぎ | 6業務 × 固定/引継ぎ × 3反復 × 4方式 | 144 |
| F1 手順作成/runtime | 8課題 × expert/Luna生成 × 2runtime × 障害0/2 × 3組 | 192 |
| F2 shell/並行性 | 8課題 × 2障害 × 3反復 × structured-tools/直列化の追加2条件 | 96 |
| G 実成果物 | 2ケース × 正常/結果失敗/引継ぎ × 3反復 × 4方式 | 72 |
| 通常分の合計 | 既存A 2,592 + 追加B–G 1,368 | 3,960 |
| 予備枠 | 非採点pilotで用途と精度を事前設計する | 最大600 |
| 総上限 | protocol生成、build、証明、決定的試験、非採点接続確認は別計上 | 最大4,560 |

既存`experiment.json`とrunnerのmatrixはAのみを実行できる。追加B–Gをそこへ未実装のまま
足したり、上限4,560を採点可能件数として表示したりしない。
Bは原48課題から4family×3難易度を結果を見る前のnative表現監査で選ぶ。
AのChoreoFix/最新TraceFix各108件を再利用できない場合、予備枠から最大216件を再実行する。
再利用はsource、binary、Wasm、prompt、interface、model、config、採点がすべて一致する場合に限る。

Cは元の航空業務rewardと確認指標を保ち、対象予約・変更・現在引数への承認を独立監査する。
別の構造化承認ケースでは全方式へ同じ確認情報を与える。simulated customerの条件と利用量も揃える。
Dは反復合意、人間承認、parallel、実行時workflow計画を含める。
Eはreview→merge、build→release、専門家引継ぎ、work-pool、共有解析、再編集後の承認更新を扱う。
Gは実patchを独立testする小さい公開repoと、固定データの数値・図表・本文・出典を外部検査するreportとする。
少数ケースから一般的なcoding能力を主張しない。

F1は48生成campaign、初回+最大4修正、最初の意味適合候補を採用する。
同じ部分仕様・語彙・制御範囲を与え、固定localside・effect adapterへ適合させる。
探索範囲を揃えられなければnative生成比較に限定する。F2は分離した研究artifactとし、
coreへ互換mode、第二の権限経路、通常実装の並行性を削る制限を残さない。
変更負担は同じ8課題へ検証追加、retry規則変更、参加者関係変更を与え、
手編集・生成物・汎用engine修正を分け、熟練度の影響も記録する。

## 測定と分析の事前固定

システム全体のnative比較と、手順・effect interface・prompt情報・予算を揃えた因果比較を分ける。
比較先へ新schedulerや権限制御を足す必要がある課題はnative未対応とし、追加adapter条件を別に記録する。
可視text/function履歴、JSON表現、圧縮・切詰めを保ち、opaque reasoning stateを要求間で持ち越さない。
1run600秒、参加者200tool calls、domain delay1.0、同時run1を維持し、run内部の参加者は並行実行する。

主指標は元の業務の完全完了率。不正な実効果は独立oracleで数え、拒否、合法操作の受理、
protocol完了、domain失敗、timeout、予算切れ、並行性を併記する。
時間とinclusive tokensは失敗を含む全試行を集計し、成功例平均と成功1件当たり総費用を分ける。
cached inputを含め、reasoning tokensをoutputへ二重加算せず、利用量欠測はunknownとする。

task・障害・反復を対応付け、clusterを保つpaired bootstrapの95%区間を用いる。
Aは16family別とleave-one-family-out、Bは採用family、航空業務はtaskをclusterとする。
小さいworkflow/caseは記述的に報告し、同じ集合の主要3対照にはHolm補正を使う。
主要対照の名前と必要精度はM0で本番前に確定する。
完了率の非劣性許容差5percentage pointsは候補であり、実務上の意味と必要精度を非採点pilotで
検討して本番前に凍結する。予備枠を勝てるまで増やさず、精度不足は不確実と報告する。
0違反、有意差なし、有界探索成功を、それぞれ普遍安全性、同等性、形式証明へ読み替えない。

quota、認証/モデル不一致、壊れたadapterはinfrastructure failureとして区別する。
通常の逸脱、予算切れ、timeoutは結果とする。再実行規則は先に固定し、失敗のraw記録を残す。
保証に加え、安全性と完了率を保ち、費用・並行性・変更負担の実質的改善を狙う。
競合と差がなければ、優位性の主張を機構の統合に限定する。

## 実装上の契約

Globalを唯一の連携仕様にし、localsideへEndpointの操作を直接書く。
順序・分岐・再考・並行処理・完了をhelper、wrapper、flag、所有者表、別の状態機械へ移さない。
資源の実体を保存するbufferと、資源の操作権・進行を区別する。

- Global: `seq`、`send`、`route`、resolver、`par`、`roll`。
- Localside: `send`、`recv`、`offer`、`par`を直接記述する。`hibana-runtime`が
  借用したfutureの保存・poll・dropを担当し、通信の進行・所有権はHibana本体が検査する。
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
`e139912e07fe4cd5ba599dd3749d6e03ce9d1ee1`、WASI
`6b62998b614e2b68feecd2d94042adbcc614693d`を選択している。
executorは`hibana-runtime`の`55dca34e36fb48e9384881e50c9092822df1511c`を選択する。
通常のGit依存を使い、診断用のpath patchを認定へ流用しない。

runtimeの中核はno_std・割当なし・caller-owned storageである。12課題の起動処理から
joinツリーとguest/I/Oの二重の終了値回収を除去し、
I/O参加者がファイル内容と最終I/Oを所有して終了する形に変更した。全メッセージ型の
使用箇所で推論できるlifetimeを省略し、型定義の借用関係は維持した。
選択中のruntimeは、Futureを最終のinline slotに置いてから参照の平衡木を一度だけ構築する。
既存Futureを値で繰り返し移動するstorage構築と、入力を二重に保持するawait wrapperを除去した。
利用側のpin操作やstack増量、heap割当は追加しない。
同じ入力とstack予算で旧実装がoverflowし、修正後が通過する実行試験と感度検査を追加した。
native59試験、compile入力12件、strict-provenance Miri51試験、Lean16定理/Z3、no_std/Pico linkを
含む全runtime gateが同じsourceで成功した。Lean/Z3はschedulerとstorageの性質を扱い、
macro変換全体や任意のRustの詳細化証明ではない。

この通常Git pinで、12課題の実Wasm74条件がprotocol完了し、監査違反は0件だった。
別のCI/CD早期終了1条件では共有効果0件を確認した。記録変異1,307件をすべて拒否した。
アプリ全targetのRust36試験、runner17試験とClippy、全binaryのbuildが通過し、
アプリとrunnerの検査前後のsource hashは一致した。全子孫の最大memoryは1,525,792,768bytes。
記録は`output/review-occurrence-qualification/choreofs-controlled-concurrency`に保持する。
モデル呼出し・採点済み実験への算入は0件であり、各条件の業務結果は元scoreへ別に記録する。

1Hの通常遅延でcommitの重なりが2件になった記録は保持する。parは同時実行を許すが、
実装の完了時刻が異なる通常実行で必ず3件が重なるとはしない。独立したA/C/Eの実要求を
Endpointで受理した後、試験用I/O境界で3件がそろうまで保留する条件を別に追加した。
返答なしで3件が受理され、解放後の実commitも3件重なった。globalとlocalsideは変更していない。
この開始条件を制御する機構試験を、通常の性能測定や実モデルの並行性へ混ぜない。

Tokioは外部I/O、runner、entry pointと試験に残っており、完全除去は未完である。
次はTokioの完全除去を先に完了し、そのsourceを固定してからAI予備実験へ進む。
比較接続用の未使用bridgeとHTTP依存を削除した。元の比較実装は変更していない。

| 対象 | 現状 | 残る確認・変更 |
| --- | --- | --- |
| 1E/1M/1H | 実資源Endpointの要求・結果・返却。1Hは7人が実結果で再考。実Endpoint・実Wasm試験と実モデルの予備記録あり | source固定後の全条件の認定と比較 |
| 2E/2M/2H | 両sectionの実Endpoint要求・結果・返却。実Wasm15条件と3/5/7人のLuna予備試験が通過 | source固定後の全条件の認定と比較 |
| 6E | 実Repo Endpointによるglobal。各拒否後に修正・再レビューを行い、承認後の並行な後続へ進む | 通常Git pinで実Wasm通常26条件と追加8条件、3人の実モデル予備試験が成功。wire label・laneの独立監査と全条件の認定は未完 |
| 8E | DB・test環境の実Endpoint委譲、単独所有の結果ファイル、実Wasm3条件、Luna成功・終端失敗の予備試験 | source固定後の認定と比較 |
| 9E/9M/9H | 実fork Endpointを両方受け取り、実要求・結果・返却。3/5/7人の実Wasm・実モデル予備試験あり | source固定後の認定と比較 |
| 16E | build/deploy環境の実Endpoint委譲。実Wasm3条件、早期終了拒否、3人のLuna予備試験あり | source固定後の認定と比較 |
| 3E | nativeの研究report protocolと意味試験あり | 実Endpoint委譲・実Wasmへの移行 |
| その他の課題 | 元の48入力・825 checklist項目を照合するmanifestとmatrixあり | 各課題のglobal/localside、実Wasm、比較入力の認定 |

同じ本体pinを使っていても、別sourceの予備記録を最終source全体の認定へ読み替えない。
固定script、実モデル、元simulatorの完了、protocol完了を別々に報告する。

移行済み6familyの旧native coordinator、専用のunit貸出broker、要求待ち行列と
完了集計を除去した。置き換えられた経路だけを試すテストと古いPython試験も除去した。
委譲版のglobalとlocalsideは前版からbyte単位で変更していない。実験runnerでは記録fileを
一つのI/O threadが所有し、書き込み・flushの結果を返す。元simulatorの参照も一つのI/O
threadが所有し、非同期の元toolを並行に開始する。共有FileのMutex、simulatorのArc、
callbackのMutexは除去した。これらのI/O窓口は業務の進行や許可を判定しない。

空targetから全実験binaryをbuildし、4分54秒、最大963,675,552bytesで成功した。
compile時間の改善は確認していない。36 native試験、アプリとrunnerのClippy、runnerの
14試験が通過した。元の48課題×3障害条件の初期化、独立操作と競合の感度、tool例外の返却も
実simulatorで確認した。移行済みの課題数が48になったという意味ではない。

同じ通常Git pinとbinaryで、1E/1Mの9条件、1Hの5条件、レビュー34条件が実Wasmで成功した。
1Hは元の障害0/1/2、追加の2回失敗、通常遅延を含む。通常遅延では独立commitが3件重なり、
違反0だった。coding各記録の10変異、計140件と、レビューの826記録変異を拒否した。
これらはモデル呼出し0の機構試験で、比較の採点数は増やさない。

ChoreoFix `252b821cffcec3a5f172aa64e018db8e709cc8da`のbinaryと通常Git依存で、
1Hの7人と6Eの3人のLuna xhighを、ChatGPT subscriptionから実Wasmで実行した。
1Hは79.57秒、inclusive 1,820,465tokens。各人が実際のテストに2回失敗した後で成功し、
資源要求・返却各42件、独立commitの同時実行2件、元score完了・監査違反0だった。
6Eは30.29秒、inclusive 284,702tokens。4回のレビュー、資源要求・返却各6件で完走し、
実装/レビューの重なり3組、merge/レビューの重なり1組を観測した。
raw、各参加者のusage、binary/source receiptは
`output/canonical-model-qualification/subscription-1H-1793407820497950000`と
`subscription-6E-1793408026651117000`に保持する。
これらは2件の非採点pilotで、全課題・全条件の認定や比較上の優位性ではない。

レビュー監査は要求・close・結果の出現を一対一で照合する。ただし同じframeの全観測で
wire labelまたはlaneを一斉に変えても受理する不足を、上記6E記録で再現した。
実行した投影から取り出すwire対応表との照合は未実装で、独立監査が投影全体を保証したとは言わない。
未移行36課題と、T3/T4の最小合成を具体的な全効果入口へ適用する認定も残る。

### 以前の版の予備試験

以下は個別の移行時点の記録であり、現在の全source・全依存版の認定とは区別する。

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

両featureが拒否された後にも、他方の修正と再レビューを並行実行するようglobalとlocalsideを修正した。
現行globalのbuildは294秒、全子孫の最大memoryは1,092,634,112bytesで、
build前後のアプリ・core source hashは一致した。
元障害条件0/1/2、最初の対象A/B、一方を続ける・交互・交互の後に同じ対象を続ける3順序を
実Wasmで実行した。短縮遅延18条件と通常遅延8条件の計26条件すべてで元score完了・資源違反0件。
各16変異、合計416件を監査が拒否した。監査は実frameとWASI要求、業務引数、元simulator結果とscore、
貸出・返却、最新実装への承認後のmerge、guest終了を照合する。
通常遅延8条件すべてで実装・レビュー、merge・レビューの実時間が重なった。
両拒否後の交互順序では、修正・再レビューを含む3組以上の実装/レビューの重なりを確認した。
試験は173.38秒、最大memory479,723,520bytes。実行時の拒否試験と記録変異の検出力は別の証拠である。
アプリとrunner全targetのClippy、runner通常12試験とfmtが通過した。
26記録は別processで同じ監査を実行し、組み込み監査と一致した。二つの独立アルゴリズムとは数えない。

その後、同じ引数の再レビュー要求を削除しても旧監査が過去の要求を再利用して受理することを
実記録の変異で再現した。監査を要求・引数ファイルclose・結果の出現ごとの一対一対応に修正し、
直前版の26記録が引き続き通り、22種または25種、合計626記録変異をすべて拒否した。
これらは記録監査の検出力であり、626件の実行時攻撃試験ではない。

元の採点matrixは変えず、両feature拒否後の3回目・4回目の拒否を追加する8機構条件を作った。
4回目の拒否で、内側の反復を終えたglobalが外側へ再入し、レビュー待ちの開発者に再実装を
要求する不整合を実Wasmで再現した。globalの反復境界を修正し、localsideへの例外分岐や
操作順の回避は加えない。初回承認を明示的なrouteで受け、拒否の場合だけ修正・再レビューの
反復へ入り、承認を受けてから後続へ進むglobalへ修正した。追加8条件は実Wasmで完走し、
実効果と監査が通過した。モデル呼出しは0で、採点対象の比較実験には数えない。

source生成・lane割当の上限は、論理方向の索引、因果計算の一括化、作業領域の再利用により
解消した。初回承認の分岐を含む現在の1586event globalも、sourceと投影の意味照合を
維持したままforbidで通常buildが通過した。
正の順序義務は、activeなsource順序のcover（間に別のactive操作がない前後関係）を
すべて含む候補へ絞った。余分な直接順序の検査は全件維持し、従来の全操作対の検査との
一致を含む投影135試験が通過した。小さいsourceは全choice/holder条件からcoverを独立に
求め、候補に欠落がないことも照合する。有限な後方・推移的source順序について、これらの
検査が全祖先のcoverageとsource-closedな進行のreadinessを保つLean補題が通過した。
1586eventの通常buildは1分7秒、最大840,564,736bytesで通過した。これは増分buildで、
clean build時間ではない。修正後の空targetからのbuildは4分55秒、最大1,120,649,216bytesで
通過し、実Wasm34条件に使ったbinaryとbyte単位で一致した。forbidは維持している。
const評価の上限は解消したが、cold build時間の改善は残る。rustcの標本ではconst評価が
実行されており、Cargo timingsとprocess sampleを保持した。
その先の実Wasmで見つかった送信のOutOfOrderとofferの停止は、
小さい実Endpoint試験へ切り出し、本体で修正した。resolverの確認が受理済みの送信候補を
同labelの別の出現へ戻さないこと、別armへ移る場合も送信方向・logical role・payloadを
照合することを守る。offerは、全祖先の確定した選択から送れる候補を、追加frameを待つ
候補より先に提示する。実受信frameの選択と、後段のadmission/commit検査は維持する。
localsideの操作順・通信・並行性に回避処理は加えていない。
修正coreを使う追加8条件は、3回目・4回目の拒否を含めて成功し、監査違反0、各記録の
25変異、計200変異をすべて拒否した。同じbinaryで通常26条件も完走し、違反0、626記録変異を
すべて拒否した。通常遅延8条件では、実装/レビューとmerge/レビューの実効果の重なりも確認した。
新しい送信/offerの3回帰試験はstrict-provenance Miriを通過し、候補順位の本番関数はKaniで
検査した。送信候補とsourceの対応、無frameで送れる候補の優先規則に関するLeanとZ3も通過した。
送信の修正だけを外す変異は実Endpointの2試験、offerの修正だけを外す変異も2試験で失敗し、
元のsourceへ戻すと3試験すべてが通過した。検査前後のsource hashも照合した。
その後、coreを`e139912e`、WASI runtimeを`6b62998b`へ固定し、ローカルpatchなしで
同じ1586eventを再buildした。4分53秒、最大1,104,922,112bytesで通過し、依存するcoreは
一つだけであることを確認した。このbinaryで通常26条件と追加8条件を再実行し、全34条件が
成功、監査違反0、826記録変異をすべて拒否した。build前後のsource hashと実行binaryのhashを
記録した。228.39秒、最大75,726,848bytesで、モデル呼出しは0、採点実験には数えない。
WASIのローカルgateも通常Git pinで通過した。実Wasmのシェル例、LeanとZ3、Pico 2の
resource link、Miri、doc、packageを含む。Wasm専用のguest sourceをhost向けのCargo exampleと
して自動検出しないようにし、通常のcargo test、Clippy、packageも確認した。
現版のモデル認定は残る。

修正coreはlib817試験、roll継続再利用13試験、Clippy、no_std/ARM gateを通過した。
cover変更後のLean全127moduleと全1233定理の公理監査、Z3の14query、依存関係の変異を
拒否するMiri1試験、word/filter/intersectionのKani3件も通過した。Kani3件はword演算の検査で、
cover候補の再帰導出全体を証明したものではない。分岐履歴のMiri2件と方向索引のMiri1件、
全u8域の方向索引・parallel候補のKani2件、条件付き依存の上下界に関するLeanとZ3も通過した。
これらをcore全ゲートやT3/T4の合成の完了としては扱わない。

前版の3人のLuna xhighは通常遅延・障害条件2で39.13秒で完走した。両featureが一度ずつ拒否された後に
成功し、元score完了、違反0件、実Endpoint要求・返却は各6件だった。全員のChatGPT認証、モデル、
effort、usageを確認し、inclusive tokensは360,770だった。実モデルの記録も16変異を拒否した。
16個のscript記録と1個のモデル記録は別processでも同じauditorを実行し、組み込み監査と一致した。
監査アルゴリズム自体は同じである。使用binary、Wasm、runtime source、モデル記録を保持した。
これらはローカルcore修正を選ぶ予備試験である。前版のモデル記録を変更後sourceのモデル認定へ
流用しない。当時は通常Git pinと変更後sourceのモデル接続確認が未完だった。
現在の1H・6Eの確認範囲は上段の記録に従い、これらの旧試験を採点済みとは数えない。

効果入口の契約と12課題・6familyの受信処理を[一覧](benchmark/effect-boundaries.md)に整理した。
6Eの実受信処理を別runtime間で直接動かし、引数のすり替え・対象違いの拒否と、
正しい要求の実結果の返却を確認した。16Eでは、承認した引数と異なる要求を受け取っても
実効果を呼んでいた受信側の不足を再現し、3つのCI/CD処理を修正した。文章作成も、
2つの資源要求の一致だけでなく、承認済み引数との一致を受信側で確認する。
比較用に残すのは実入力の値であり、進行flagや別の権限表は追加していない。

修正後のnative19件とClippyが通過した。実Wasmは16Eの元障害3条件・作業前の終了拒否と、
2E/2M/2Hの各5条件、計19条件が通過した。資源違反0件、監査の252変異を拒否し、
文章作成の終端レビュー失敗3件は元scoreの失敗として保った。通常遅延の独立writeは
3/5/7人で1/2/3件重なった。完走した18記録を別processの同じauditorでも照合し、
組み込み監査と一致した。監査アルゴリズムは同じである。
build前後のアプリ・core・WASI・washのsource hashは一致した。
buildは306秒、全子孫の最大memoryは1,088,407,040bytesだった。
これらはローカルcore修正での機構試験で、model呼出し・採点実験は0件である。
この19条件を現在の通常Git pinで認定し直す作業と、T3/T4の一般的な合成は残る。

1Hの委譲版には、通常のmodule localsideを動かさず、現在のglobalを公開`project`で
直接駆動する排他試験を追加した。A/Bそれぞれのdesign・implementationを完了し、
AがDB用Endpointを保持する間はBの受信がPendingとなり、Aの実要求・結果・返却後に
同じEndpointをBが取得できる。保持中の別attachも両者について拒否される。
この1つの操作列はローカルcore修正を選ぶmodule単位の試験で通過した。
全スケジュールの証明や通常Git pinでの全crate認定としては数えない。

旧native RPC、専用CLIと小型shellは削除済み。実験用transportは一つの値がqueueとbindingを
所有し、runtimeが借用する。現行の外部JSON接続は`ExternalIo`がstdin/stdoutと応答対応を所有する。
API結果ファイルはファイル担当が内容・offsetを所有し、metadataもglobalの要求・応答で処理する。
旧native codebaseの要求queueとrunnerの共有記録・callbackは`252b821`で除去した。
後続でAPI／codebaseの旧Python接続も除去した。未移行課題は現行の実Wasm経路へ移す。

通常の実験コマンドとsubscription試験は、同じrunnerから移行済み12課題を実行する。
選択したruntimeのbuild receiptと全sourceを照合し、実行前にbinary、wash、入力を保存する。
この入口から6E・障害条件2をLuna xhigh 3人で実行し、43.02秒で元課題を完了した。
実モデル3人の重なり、review 4回、実装／reviewの重なり3組、merge／reviewの重なり1組、
違反0件、inclusive 426,541 tokensを独立監査で確認した。採点比較への算入は0件である。
監査器のthread応答IDの決め打ちを、記録されたrequestとの対応へ修正した。
同じraw記録で監査と9種類の記録変異の拒否を確認し、モデルは再実行していない。
runnerの通常テスト24件、Clippy、buildが通過した。wire label／laneの投影照合は引き続き未完である。

食事課題も共通のRust runnerから実Wasm・元simulator・記録監査を実行する形へ統一した。
3／5／7人の3条件を通常遅延で完走し、独立した食事の最大重複は1／2／3件、違反は0件だった。
早すぎる要求、重複、型違い、分割書込み、表示名引数の保存、両資源の返却を検査し、
9種類×3記録の変異を拒否した。モデル呼出しは0件で、試験全体は54.25秒だった。
globalの構成とEndpoint操作順は変更していない。roleとsessionの定数を監査と共用し、
専用のPython subprocess、別監査binary、未使用の旧接続・監査scriptを削除した。
この時点で残っていたAPI／codebaseの旧CLIは、下記のRust経路への統一で除去した。

同じ通常Git pinのbinaryを使い、9HのLuna xhigh 7人による非採点pilotを一度実行した。
19.784秒で元課題を完了し、実モデル7人と独立した食事3件の重なり、14本のEndpoint返却、
違反0件、inclusive 571,238 tokensを独立監査で確認した。人工的なbarrierは使っていない。
実記録の10種類の変異を拒否し、raw model／shell／transport／元採点、sourceとbinary hashを保存した。
runtimeのbuild前後のsource一致、アプリ21試験、runner24試験、両者のClippyを確認した。
このpilotと機構試験は採点比較へ数えず、wire label／laneの投影との意味照合は未完のままである。

APIとcodebaseの実Wasm機構試験も、通常runnerと同じRust simulator接続・監査・
source保存へ統一し、旧CLIと最後の自作Python接続を削除した。元simulatorは変更していない。
APIは3条件・22.45秒、codebaseは14条件・415.52秒で成功し、記録変異42件・140件を拒否した。
APIではDB_ADMIN／BACKEND_DEVの結果file読取りを両方Pendingにした後にTESTERを開始し、
結果の単独所有と独立した私有I/Oを確認した。これはモデルなしの明示的な試験用scheduleである。
codebaseは1Hの再考、1E／1Mの正常な業務失敗、早すぎるcommit、引数・型・重複の拒否、
独立commitの並行性を保つ。全17条件で実行前にruntime、wash、runner、source／receiptを保存した。
アプリ21試験、runner24試験、両者のClippy・build、実行前後のsource一致が成功した。
全子孫の最大memoryは704,987,136bytes。モデル呼出しと採点比較への算入は0件である。
記録は`output/review-occurrence-qualification/rust-boundary-unification`に保持する。

通常runnerの終了判定で、内側のparticipant結果に保存したtimeout／接続errorを
集約しない欠陥を修正した。guestの完了後でもモデル参加者が失敗すれば、記録のerrorと
CLIの失敗終了へ反映する。runtimeのerror・異常終了・backend失敗も同じ集約で確認する。
正常に完了したprotocolとモデルが返す業務失敗は、引き続き元scoreの失敗として区別する。
実際のbyte-stream driverへ成功したguest終了応答を返した後、接続切断／timeoutを起こす
2回帰試験が成功した。旧集約だけへ戻すと両試験が失敗し、復元後はrunner全28試験・
Clippy・buildが成功した。最大memoryは705,511,424bytes、モデル呼出し0件だった。
記録は`output/review-occurrence-qualification/participant-result-qualification`に保持する。
全員の完了報告が揃った後のgroup errorも失わず、通常runnerへ返すよう修正した。
未完了RPCを残して全員の完了通知を返すbyte-stream試験で、この経路の拒否を確認した。
この修正だけを戻すと回帰試験が失敗し、復元後の全29試験・Clippy・buildが成功した。
この追加検査の最大memoryは701,235,200bytes、モデル呼出し0件である。

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

GitHub Actionsの残予算はなく、最終gateはローカルで確認する。
core `967a89bbbbb392387e70a74b579ecf39ec21b5c5`ではKani全319件が成功し、
全子孫の最大memoryは2,647,310,336bytesだった。元の記号入力とunwinding検査を維持した。
独立していた所有権の4証明を同じ検証入口へ統合した。本体の実行挙動は変更していない。
その全gateは、入れ子5選択の生成ケース`spines 5/22409`のRust compileがmemory上限を超えて停止した。
`d5b32c2a44c1c0dc30573425007dfe2be1ff1b59`で投影作成を同期関数へ分け、型付きglobalを
試験のasync frameへ保持しないよう生成器を修正した。本体の実行コードは変更していない。
同じ3契約の構文・操作text・probe一覧が不変であることを照合し、36完走・1,092呼出し・
2,043,136操作試行でモデルと実行時の不一致0、2,240局所拒否、全感度controlが成功した。
最大memoryは2,731,363,112bytesで、元と同じ2.75GB guard、forbid、最適化とassertionを維持した。
この版のKani全319件は成功し、全子孫の最大memoryは2,587,378,000bytesだった。
続く生成検査では、同じ公開message contractのaliasからresolver入力を作り直すと、
受理時と異なる分岐になる試験器の不具合が露呈した。`510fe108`で受理時の入力を保存し、
再生とLean照合で同じ入力を使うよう修正した。元の停止shardは188形・681,484操作試行で
runtime-only/model-onlyの不一致0だった。元の試験器の処理だけを戻すと回帰試験が失敗する。
同版の全gateはrolled predecessorのKaniで2.75GB上限を超え停止した。
`bb33e40c1e7e7f5d7b338e41b0b7448ef10f3eb3`では元の`u8 % 3`の値域を0/1/2へ分割し、
他の記号入力とassertionを維持した。個別の3証明は成功したが、最終sourceのKani全件と
protocol fuzz、Miri等を含む全gateは再実行が必要であり、全体成功とは扱わない。
アプリの通常Git pinは`e139912e`のままで、固定する本体版への更新と統合認定は残っている。
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

| 工程 | 次の成果物と通過条件 | 状態 |
| --- | --- | --- |
| M0 比較の固定 | 基準SHA、構文範囲、native表現監査、課題と接続、指標・精度・再実行規則を固定する | 更新計画を記録。新競合と精度設計は未認定 |
| M1 意味論 | T1–T6の定義・前提・反例・必要補題と、既存core定理の対応表。評価に使うpar/roll/委譲を含む | T3/T4の最小receiver契約下の合成済み。残る命題の対応と、具体的な二資源・実効果への適用を進める |
| M2 実装と証明 | 効果入口一覧、実行時の負例、Leanと実装照合、必要なcore/WASI gate。通常Git pinでレビューと1Hを認定する | 通常Git pinの実Wasmはレビュー34条件・1H5条件・食事3条件、実モデルは1H7人・6E3人・9H7人のpilotが成功。wire監査、全条件・全効果入口とT3/T4の一般合成は未完 |
| M3 課題と接続 | 全48課題と採用追加課題のglobal/localside・実Wasm・元採点・接続を認定する | 12課題を移行済み。残る36課題を移行する。比較接続の開発は保留 |
| M4 本比較 | A1の864件を先に実行し、A2とB–Eを認定後に実行する。全identityにrawと終了理由を残す | 採点0件 |
| M5 原因と分析 | F/G、8課題の変更負担、cluster対応集計、負例と不確実性を整理する | 未実行 |
| M6 論文化 | 最新owned実装へ草稿を同期し、各主張と定理・測定・TCB・再現artifactを対応付ける | 草稿。結果表と結論は未確定 |

直近の作業順は、(1)本体の最終sourceで全gateを完了、(2)T3/T4と全効果入口・wire監査の対応、
(3)通常Git pinの更新と移行済み課題の実験・残る課題の移行、
(4)T1/T2へpar/rollを接続しT5/T6の前提を固定、とする。比較接続の再現作業は進めない。
M1と効果境界を最初の技術判断点にし、
資源の合成保証が成立しないまま2,592件を先に消化しない。独立したM3の作業は進めてよい。

M3では旧nativeの進行管理・不要な入口・自作Pythonを除去し、残る課題を委譲と実Wasmへ
移行する。比較接続の新規開発と応答境界の再現作業は行わない。ChoreoFixの実験は
利用可能な接続条件で進め、source、model、設定と実行条件を保存する。
比較対象がその条件で動かない場合は未実行と記録し、ChoreoFixの実験を止めない。
元の比較sourceを変更せず、条件の違いがある結果を同一条件の比較として報告しない。

各認定にはsource/lock/proof/compiler/binary/Wasm/model/configとbuild receiptを保存する。
古いsnapshotの`.cargo/config.toml`前提やauditの呼出し数も最終経路で検査する。
正常・拒否・故障・早期終了、実資源の唯一性と結果、実際の並行性、oracleの感度を確認する。
旧成功を変更後sourceへ流用せず、具体的な不一致の修正を試験拡大より優先する。

受領計画の技術予算8–12人週は納期の約束ではなく、M1で再見積もりする。
全runが600秒を使う逐次実行の上限はAが432時間、最大4,560件で760時間である。
実際の所要時間は非採点pilotの分布・接続制限・再試行から見積もり、上限計算と区別する。
予備試験の成功を成功率・速度・品質の優位性へ外挿しない。

## ローカル作業と完了判定

重い検証は同時に一つ、Cargo/Kani jobsは1。全子孫合計3,000,000,000bytesで停止するguardを使う。
現在は余裕を取り2,750,000,000bytesのguardで実行する。上限停止は未検証として残し、
予算の引上げ、globalの縮小、検査の無効化で成功へ変えない。検証前後のsource hashを照合する。
最新stableで開発し、Miri/Kaniだけは各ツールが必要な検証compilerを使う。

既存のdirty sourceを保ち、同名packageの別checkoutでtarget directoryを共有しない。
本体等の検証済み修正は開発branchへpushする。研究repoの現在sourceはGitHubの
`development/endpoint-delegation`へ公開し、旧検証ログ・sourceコピー・build生成物は追加しない。
GitHub Actionsの残予算はないため、必要な検査はローカルで実行し、進捗pushは`[skip ci]`を付ける。
古いbuild・不要cache・未使用の実験成果物は削除し、profilerと論文に必要なraw証拠は保持する。
ビルド成功後に毎回この整理を行い、置き換えたbinaryや未使用target、一時ログを溜めない。
現在使う成果物・再利用するcacheと、未解決の反例・検証停止の証拠は残す。圧縮保管へ逃がさない。

完了にはT1–T6と実装の対応、受理した全課題の適合性、実際に使う接続条件の記録、
事前に凍結したA–Gの実験・負例・独立採点・集計、論文と再現artifact、必要な統合gateが必要である。
未対応と除外母集団を明記する。最大600の予備枠を使い切ること自体は完了条件にしない。
未実行・間接的な証拠は完了に数えず、全Rust・全分散障害の完全証明を新たな無期限の条件にしない。

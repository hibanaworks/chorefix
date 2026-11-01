use super::{global::*, imp::OutcomeFile, localside};
use crate::{imp::transport::InMemoryTransport, shell::ShellResult};
use hibana::{
    g,
    runtime::{SessionKitStorage, ids::SessionId, program::project},
};
use hibana_wasip1_runtime::protocol;

#[hibana_runtime::main]
#[test]
async fn metadata_is_served_by_the_file_owner_before_and_after_the_result() -> ShellResult<()> {
    let publication = g::seq(
        g::par(
            g::route(
                g::send::<TESTER, ADMIN_OUTCOME, PassedAdmin>(),
                g::send::<TESTER, ADMIN_OUTCOME, FailedAdmin>(),
            ),
            g::send::<ADMIN_OUTCOME_VM, ADMIN_OUTCOME, OutcomeOpen>(),
        ),
        g::seq(
            g::send::<ADMIN_OUTCOME, ADMIN_OUTCOME_VM, OutcomeOpened>(),
            g::route(
                g::seq(
                    g::send::<ADMIN_OUTCOME_VM, ADMIN_OUTCOME, OutcomeRead>(),
                    g::send::<ADMIN_OUTCOME, ADMIN_OUTCOME_VM, OutcomeReadDone>(),
                ),
                g::seq(
                    g::send::<ADMIN_OUTCOME_VM, ADMIN_OUTCOME, OutcomeStat>(),
                    g::send::<ADMIN_OUTCOME, ADMIN_OUTCOME_VM, OutcomeStatDone>(),
                ),
            )
            .roll(),
        ),
    );
    let metadata = g::route(
        g::seq(
            g::send::<ADMIN_IO, ADMIN_OUTCOME, OutcomePathStat>(),
            g::send::<ADMIN_OUTCOME, ADMIN_IO, OutcomePathStatDone>(),
        ),
        g::send::<ADMIN_IO, ADMIN_OUTCOME, OutcomeExit>(),
    )
    .roll();
    let global = g::par(publication, metadata);
    let owner_program = project::<ADMIN_OUTCOME>(&global);
    let vm_program = project::<ADMIN_OUTCOME_VM>(&global);
    let io_program = project::<ADMIN_IO>(&global);
    let tester_program = project::<TESTER>(&global);
    for success in [true, false] {
        let transport = InMemoryTransport::new([]);
        let mut slab = vec![0; 32 * 1024];
        let mut storage = SessionKitStorage::uninit();
        let runtime = storage.init().rendezvous(&mut slab, &transport)?;
        let session = SessionId::new(28);
        let mut owner = runtime.enter(session, &owner_program)?;
        let mut vm = runtime.enter(session, &vm_program)?;
        let mut io = runtime.enter(session, &io_program)?;
        let mut tester = runtime.enter(session, &tester_program)?;
        let mut file = OutcomeFile::new();
        let bytes =
            serde_json::to_vec(&serde_json::json!({"success":success})).expect("test result JSON");
        hibana_runtime::race(
            async {
                Ok::<_, std::io::Error>(
                    async {
                        async {
                            async {
                                (localside::outcome::admin::admin(&mut owner, &mut file)).await
                            };
                            async {
                                let stat = protocol::PathFilestatGetReq(
                                    protocol::PathFilestatGet::new(3, 0, OUTCOME_PATH)?,
                                );
                                io.send::<OutcomePathStat>(&stat).await?;
                                let empty = io.recv::<OutcomePathStatDone>().await?;
                                assert_eq!(empty.0.errno(), 0);
                                assert_eq!(empty.0.size(), 0);
                                let open = protocol::PathOpenReq(protocol::PathOpen::new(
                                    3,
                                    1 << 1,
                                    OUTCOME_PATH,
                                )?);
                                vm.send::<OutcomeOpen>(&open).await?;
                                io.send::<OutcomePathStat>(&stat).await?;
                                let waiting = io.recv::<OutcomePathStatDone>().await?;
                                assert_eq!(waiting.0.errno(), 0);
                                assert_eq!(waiting.0.size(), 0);
                                if success {
                                    tester.send::<PassedAdmin>(&bytes.as_slice()).await?;
                                } else {
                                    tester.send::<FailedAdmin>(&bytes.as_slice()).await?;
                                }
                                loop {
                                    io.send::<OutcomePathStat>(&stat).await?;
                                    let published = io.recv::<OutcomePathStatDone>().await?;
                                    assert_eq!(published.0.errno(), 0);
                                    if published.0.size() == bytes.len() as u64 {
                                        break;
                                    }
                                    assert_eq!(published.0.size(), 0);
                                    hibana_runtime::yield_now().await;
                                }
                                let opened = vm.recv::<OutcomeOpened>().await?;
                                assert_eq!(opened.0.fd(), OUTCOME_FD);
                                assert_eq!(opened.0.errno(), 0);
                                io.send::<OutcomePathStat>(&stat).await?;
                                let ready = io.recv::<OutcomePathStatDone>().await?;
                                assert_eq!(ready.0.errno(), 0);
                                assert_eq!(ready.0.size(), bytes.len() as u64);
                                let read =
                                    protocol::FdReadReq(protocol::FdRead::new(OUTCOME_FD, 64)?);
                                vm.send::<OutcomeRead>(&read).await?;
                                let content = vm.recv::<OutcomeReadDone>().await?;
                                assert_eq!(content.0.errno(), 0);
                                assert_eq!(content.0.as_bytes(), bytes);
                                io.send::<OutcomeExit>(&0).await?;
                                Ok::<_, crate::shell::ShellError>(())
                            };
                        }
                        .await?;
                        Ok::<_, crate::shell::ShellError>(())
                    }
                    .await,
                )
            },
            hibana_runtime::host::deadline(std::time::Duration::from_secs(3)),
        )
        .await
        .unwrap_or_else(|_| panic!("file owner stalled: {:?}", transport.take_observations()))?;
        assert_eq!(file.bytes, bytes);
    }
    Ok(())
}

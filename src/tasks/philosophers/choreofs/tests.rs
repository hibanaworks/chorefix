//! Protocol execution checks. Real shell execution is qualified separately.
use super::{
    global::*,
    identity::{self, roles9e},
    imp::PhilosopherArguments,
    localside,
};
use crate::shell::{
    ShellResult,
    global::Exit,
    imp::{bus::Bus, service::Domain},
};
use hibana::{
    Endpoint,
    runtime::{SessionKitStorage, ids::SessionId},
};
use std::cell::Cell;

#[derive(Default)]
struct Effects(
    Cell<Vec<(&'static str, &'static str)>>,
    Cell<Vec<&'static str>>,
    Cell<usize>,
);
impl Domain for Effects {
    async fn execute(
        &self,
        agent: &'static str,
        tool: &'static str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        assert_eq!(arguments["philosopher"], agent);
        let mut events = self.0.take();
        events.push((agent, tool));
        self.0.set(events);
        if tool == "eat" {
            let mut active = self.1.take();
            active.push(agent);
            self.2.set(self.2.get().max(active.len()));
            self.1.set(active);
            hibana_runtime::yield_now().await;
            let mut active = self.1.take();
            active.retain(|who| *who != agent);
            self.1.set(active);
        }
        Ok(serde_json::json!({"success":true,"agent":agent,"tool":tool}))
    }
}

#[hibana_runtime::scope]
async fn client<const ROLE: u8, const LEFT: u8, const RIGHT: u8>(
    endpoint: &mut Endpoint<'_, ROLE>,
    agent: &'static str,
) -> ShellResult<()> {
    let [mut task, mut private] = endpoint.par()?;
    let arguments = PhilosopherArguments::from_value(serde_json::json!({"philosopher":agent}))?;
    task.send::<Think>(&arguments).await?;
    let thought = task.recv::<Thought>().await?.to_vec();
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&thought).unwrap()["tool"],
        "think"
    );
    let (mut left, mut right) = {
        let [mut left_loan, mut right_loan] = task.par()?;
        async {
            async {
                let mut resource = left_loan.recv::<Loan<LEFT>>().await?;
                resource.send::<Eat<LEFT>>(&arguments).await?;
                Ok(resource)
            };
            async {
                let mut resource = right_loan.recv::<Loan<RIGHT>>().await?;
                resource.send::<Eat<RIGHT>>(&arguments).await?;
                Ok(resource)
            };
        }
        .await?
    };

    let (left_result, right_result) = (async {
        async { Ok(left.recv::<Meal<LEFT>>().await?.to_vec()) };
        async { Ok(right.recv::<Meal<RIGHT>>().await?.to_vec()) };
    })
    .await?;
    assert_eq!(left_result, right_result);
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&left_result).unwrap()["tool"],
        "eat"
    );
    {
        let [mut left_return, mut right_return] = task.par()?;
        async {
            async {
                left_return.send::<Loan<LEFT>>(left).await?;
            };
            async {
                right_return.send::<Loan<RIGHT>>(right).await?;
            };
            Ok(())
        }
        .await?;
    }
    task.send::<ServiceExit>(&0).await?;
    private.send::<Exit>(&0).await?;
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn three_guests_use_both_real_fork_endpoints_and_return_them() -> ShellResult<()> {
    use roles9e::*;
    let programs = project_9e();
    let session = SessionId::new(identity::SESSION);
    let (bus, _reader, _terminals) = Bus::open(
        ["PHILOSOPHER_A", "PHILOSOPHER_B", "PHILOSOPHER_C"],
        [
            (session, FORK_A_RESOURCE),
            (session, FORK_B_RESOURCE),
            (session, FORK_C_RESOURCE),
        ],
    );
    let effects = Effects::default();
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let runtime = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut a = runtime.enter(session, &programs.a)?;
    let mut b = runtime.enter(session, &programs.b)?;
    let mut c = runtime.enter(session, &programs.c)?;
    let mut fork_a = runtime.enter(session, &programs.fork_a)?;
    let mut fork_b = runtime.enter(session, &programs.fork_b)?;
    let mut fork_c = runtime.enter(session, &programs.fork_c)?;
    let resource_a = runtime.enter(session, &programs.resource_a)?;
    let resource_b = runtime.enter(session, &programs.resource_b)?;
    let resource_c = runtime.enter(session, &programs.resource_c)?;
    let mut a_wasi = runtime.enter(session, &programs.a_wasi)?;
    let mut b_wasi = runtime.enter(session, &programs.b_wasi)?;
    let mut c_wasi = runtime.enter(session, &programs.c_wasi)?;
    let mut a_io = runtime.enter(session, &programs.a_io)?;
    let mut b_io = runtime.enter(session, &programs.b_io)?;
    let mut c_io = runtime.enter(session, &programs.c_io)?;
    async {
        async {
            localside::philosopher::run::<A, FORK_A_RESOURCE, FORK_C_RESOURCE>(
                "PHILOSOPHER_A",
                &mut a,
                &effects,
            )
            .await?;
        };
        async {
            localside::philosopher::run::<B, FORK_B_RESOURCE, FORK_A_RESOURCE>(
                "PHILOSOPHER_B",
                &mut b,
                &effects,
            )
            .await?;
        };
        async {
            localside::philosopher::run::<C, FORK_C_RESOURCE, FORK_B_RESOURCE>(
                "PHILOSOPHER_C",
                &mut c,
                &effects,
            )
            .await?;
        };
        async {
            localside::fork::run::<FORK_A, FORK_A_RESOURCE>(&mut fork_a, resource_a).await?;
        };
        async {
            localside::fork::run::<FORK_B, FORK_B_RESOURCE>(&mut fork_b, resource_b).await?;
        };
        async {
            localside::fork::run::<FORK_C, FORK_C_RESOURCE>(&mut fork_c, resource_c).await?;
        };
        async {
            client::<A_WASI, FORK_A_RESOURCE, FORK_C_RESOURCE>(&mut a_wasi, "PHILOSOPHER_A")
                .await?;
        };
        async {
            client::<B_WASI, FORK_B_RESOURCE, FORK_A_RESOURCE>(&mut b_wasi, "PHILOSOPHER_B")
                .await?;
        };
        async {
            client::<C_WASI, FORK_C_RESOURCE, FORK_B_RESOURCE>(&mut c_wasi, "PHILOSOPHER_C")
                .await?;
        };
        async {
            assert_eq!(a_io.offer().await?.recv::<Exit>().await?, 0);
        };
        async {
            assert_eq!(b_io.offer().await?.recv::<Exit>().await?, 0);
        };
        async {
            assert_eq!(c_io.offer().await?.recv::<Exit>().await?, 0);
        };
        Ok(())
    }
    .await?;
    let effects = effects.0.take();
    for agent in ["PHILOSOPHER_A", "PHILOSOPHER_B", "PHILOSOPHER_C"] {
        assert_eq!(
            effects
                .iter()
                .filter(|entry| **entry == (agent, "think"))
                .count(),
            1
        );
        assert_eq!(
            effects
                .iter()
                .filter(|entry| **entry == (agent, "eat"))
                .count(),
            1
        );
    }
    assert_eq!(effects.len(), 6);
    Ok(())
}

// Attach each fixture role and declare its production participant.
macro_rules! larger_fixture {
    ($name:ident, $project:ident, $roles:ident;
      $(($service:ident, $role:ident, $fork:ident, $owner:ident,
         $resource:ident, $resource_role:ident, $vm:ident, $vm_role:ident,
         $io:ident, $io_role:ident, $left:ident, $right:ident, $agent:literal)),+ $(,)?) => {
        #[hibana_runtime::scope]
        async fn $name(domain: &impl Domain) -> ShellResult<()> {
            use super::identity::$roles::*;
            let programs = $project();
            let session = SessionId::new(identity::SESSION);
            let (bus, _reader, _terminals) = Bus::open(
                [$($agent),+], [$( (session, $resource_role) ),+],
            );
            let mut slab = vec![0; 2 * 1024 * 1024];
            let mut storage = SessionKitStorage::uninit();
            let runtime = storage.init().rendezvous(&mut slab, &bus.transport)?;
            $(
                let mut $service = runtime.enter(session, &programs.$service)?;
                let mut $fork = runtime.enter(session, &programs.$fork)?;
                let $resource = runtime.enter(session, &programs.$resource)?;
                let mut $vm = runtime.enter(session, &programs.$vm)?;
                let mut $io = runtime.enter(session, &programs.$io)?;
            )+
            $(
                async { localside::philosopher::run::<$role, $left, $right>($agent, &mut $service, domain).await?; };
                async { localside::fork::run::<$owner, $resource_role>(&mut $fork, $resource).await?; };
                async { client::<$vm_role, $left, $right>(&mut $vm, $agent).await?; };
                async { assert_eq!($io.offer().await?.recv::<Exit>().await?, 0); };
            )+
            Ok(())
        }
    };
}

larger_fixture!(drive_five, project_9m, roles9m;
    (a, A, fork_a, FORK_A, resource_a, FORK_A_RESOURCE, a_wasi, A_WASI, a_io, A_IO, FORK_A_RESOURCE, FORK_E_RESOURCE, "PHILOSOPHER_A"),
    (b, B, fork_b, FORK_B, resource_b, FORK_B_RESOURCE, b_wasi, B_WASI, b_io, B_IO, FORK_B_RESOURCE, FORK_A_RESOURCE, "PHILOSOPHER_B"),
    (c, C, fork_c, FORK_C, resource_c, FORK_C_RESOURCE, c_wasi, C_WASI, c_io, C_IO, FORK_C_RESOURCE, FORK_B_RESOURCE, "PHILOSOPHER_C"),
    (d, D, fork_d, FORK_D, resource_d, FORK_D_RESOURCE, d_wasi, D_WASI, d_io, D_IO, FORK_D_RESOURCE, FORK_C_RESOURCE, "PHILOSOPHER_D"),
    (e, E, fork_e, FORK_E, resource_e, FORK_E_RESOURCE, e_wasi, E_WASI, e_io, E_IO, FORK_E_RESOURCE, FORK_D_RESOURCE, "PHILOSOPHER_E"),
);

larger_fixture!(drive_seven, project_9h, roles9h;
    (a, A, fork_a, FORK_A, resource_a, FORK_A_RESOURCE, a_wasi, A_WASI, a_io, A_IO, FORK_A_RESOURCE, FORK_G_RESOURCE, "PHILOSOPHER_A"),
    (b, B, fork_b, FORK_B, resource_b, FORK_B_RESOURCE, b_wasi, B_WASI, b_io, B_IO, FORK_B_RESOURCE, FORK_A_RESOURCE, "PHILOSOPHER_B"),
    (c, C, fork_c, FORK_C, resource_c, FORK_C_RESOURCE, c_wasi, C_WASI, c_io, C_IO, FORK_C_RESOURCE, FORK_B_RESOURCE, "PHILOSOPHER_C"),
    (d, D, fork_d, FORK_D, resource_d, FORK_D_RESOURCE, d_wasi, D_WASI, d_io, D_IO, FORK_D_RESOURCE, FORK_C_RESOURCE, "PHILOSOPHER_D"),
    (e, E, fork_e, FORK_E, resource_e, FORK_E_RESOURCE, e_wasi, E_WASI, e_io, E_IO, FORK_E_RESOURCE, FORK_D_RESOURCE, "PHILOSOPHER_E"),
    (f, F, fork_f, FORK_F, resource_f, FORK_F_RESOURCE, f_wasi, F_WASI, f_io, F_IO, FORK_F_RESOURCE, FORK_E_RESOURCE, "PHILOSOPHER_F"),
    (g, G, fork_g, FORK_G, resource_g, FORK_G_RESOURCE, g_wasi, G_WASI, g_io, G_IO, FORK_G_RESOURCE, FORK_F_RESOURCE, "PHILOSOPHER_G"),
);

#[hibana_runtime::main]
#[test]
async fn five_guests_keep_two_disjoint_meals_running() -> ShellResult<()> {
    let effects = Effects::default();
    drive_five(&effects).await?;
    assert_eq!(effects.0.take().len(), 10);
    assert_eq!(effects.2.get(), 2);
    assert!(effects.1.take().is_empty());
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn seven_guests_keep_three_disjoint_meals_running() -> ShellResult<()> {
    let effects = Effects::default();
    drive_seven(&effects).await?;
    assert_eq!(effects.0.take().len(), 14);
    assert_eq!(effects.2.get(), 3);
    assert!(effects.1.take().is_empty());
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn a_delayed_meal_does_not_hold_an_unrelated_fork_pair() -> ShellResult<()> {
    struct DelayA {
        receiver: Cell<Option<hibana_runtime::host::Io>>,
        sender: Cell<Option<hibana_runtime::host::Notification>>,
    }
    impl Domain for DelayA {
        async fn execute(
            &self,
            agent: &'static str,
            tool: &'static str,
            _: serde_json::Value,
        ) -> Result<serde_json::Value, String> {
            // A controlled test barrier, never a production permission.
            if tool == "eat" && agent == "PHILOSOPHER_A" {
                let mut receiver = self.receiver.take().unwrap();
                let mut byte = [0; 1];
                if receiver.read(&mut byte).await.map_err(|e| e.to_string())? != 1 {
                    return Err("delayed meal notification closed".into());
                }
            }
            if tool == "eat" && agent == "PHILOSOPHER_D" {
                self.sender
                    .take()
                    .unwrap()
                    .notify()
                    .map_err(|e| e.to_string())?;
            }
            Ok(serde_json::json!({"success":true,"tool":tool}))
        }
    }
    let (sender, receiver) = hibana_runtime::host::notification()?;
    let domain = DelayA {
        receiver: Cell::new(Some(receiver)),
        sender: Cell::new(Some(sender)),
    };
    hibana_runtime::race(
        drive_five(&domain),
        hibana_runtime::host::deadline(std::time::Duration::from_secs(5)),
    )
    .await
    .expect("D's disjoint fork pair must progress while A is delayed");
    Ok(())
}

#[hibana_runtime::main]
#[test]
async fn the_guest_cannot_send_an_eat_request_without_the_resource_endpoints() -> ShellResult<()> {
    use roles9e::*;
    let programs = project_9e();
    let session = SessionId::new(identity::SESSION);
    let (bus, _reader, _terminals) = Bus::open(
        ["PHILOSOPHER_A"],
        [(session, FORK_A_RESOURCE), (session, FORK_C_RESOURCE)],
    );
    let mut slab = vec![0; 2 * 1024 * 1024];
    let mut storage = SessionKitStorage::uninit();
    let runtime = storage.init().rendezvous(&mut slab, &bus.transport)?;
    let mut service = runtime.enter(session, &programs.a)?;
    let mut vm = runtime.enter(session, &programs.a_wasi)?;
    let effects = Effects::default();
    async {
        async {
            let [mut task, _private] = vm.par()?;
            let arguments = PhilosopherArguments::from_value(
                serde_json::json!({"philosopher":"PHILOSOPHER_A"}),
            )?;
            task.send::<Think>(&arguments).await?;
            task.recv::<Thought>().await?;
            assert!(task.send::<Eat<FORK_A_RESOURCE>>(&arguments).await.is_err());
            Ok::<_, crate::shell::ShellError>(())
        };
        async {
            let arguments = service.recv::<Think>().await?;
            let result = effects
                .execute("PHILOSOPHER_A", "think", arguments.into_value())
                .await
                .map_err(crate::shell::ShellError::message)?;
            let bytes = serde_json::to_vec(&result).unwrap();
            service.send::<Thought>(&bytes.as_slice()).await?;
            Ok::<_, crate::shell::ShellError>(())
        };
        Ok(())
    }
    .await?;
    assert_eq!(effects.0.take(), [("PHILOSOPHER_A", "think")]);
    let observations = bus.transport.take_observations();
    assert_eq!(
        observations
            .iter()
            .filter(|frame| frame["action"] == "send" && frame["source"] == A_WASI)
            .count(),
        1
    );
    assert!(
        !observations
            .iter()
            .any(|frame| frame["source"] == FORK_A_RESOURCE || frame["source"] == FORK_C_RESOURCE)
    );
    Ok(())
}

use super::{
    global::*,
    imp::{Artifact, Repository, Verification, VerificationInput},
};
use crate::shell::{ShellError, ShellResult};
use hibana::Endpoint;

pub async fn initial<'r>(
    endpoint: &mut Endpoint<'r, A>,
    mut resource: Endpoint<'r, R>,
    proposal: Artifact,
) -> ShellResult<Artifact> {
    resource.send::<Prepare>(&proposal.bytes()).await?;
    let prepared = resource.recv::<Prepared>().await?;
    resource.send::<Inspect>(&prepared).await?;
    let checked = resource.recv::<Inspected>().await?;
    endpoint.send::<Context>(&checked).await?;
    endpoint.send::<LoanR<'_>>(resource).await?;
    let branch = endpoint.offer().await?;
    match branch.label() {
        26 => {
            branch.recv::<CommitReady>().await?;
        }
        27 => {
            branch.recv::<CommitDenied>().await?;
            let _resource = endpoint.recv::<LoanR<'_>>().await?;
            return Err(ShellError::message("candidate verification failed"));
        }
        _ => return Err(ShellError::message("unexpected final verification branch")),
    }
    let mut resource = endpoint.recv::<LoanR<'_>>().await?;
    let reported = Artifact::decode(endpoint.recv::<Report>().await?);
    resource.send::<Audit>(&reported.target).await?;
    let actual = Artifact::decode(resource.recv::<Audited>().await?);
    if actual != reported {
        return Err(ShellError::message(
            "reported and published artifacts differ",
        ));
    }
    Ok(actual)
}

pub async fn recover(
    endpoint: &mut Endpoint<'_, B>,
    values: impl IntoIterator<Item = u64>,
) -> ShellResult<Artifact> {
    let context = Verification::decode(endpoint.recv::<Context>().await?)?;
    let mut resource = endpoint.recv::<LoanR<'_>>().await?;
    let mut candidate = context.artifact;
    let mut values = values.into_iter();
    let checked = loop {
        let value = values
            .next()
            .ok_or_else(|| ShellError::message("fixture has no further candidate input"))?;
        resource
            .send::<Revise>(&Artifact { value, ..candidate }.bytes())
            .await?;
        candidate = Artifact::decode(resource.recv::<Revised>().await?);
        resource.send::<Reinspect>(&candidate.bytes()).await?;
        let branch = resource.offer().await?;
        match branch.label() {
            8 => {
                break Verification::decode(branch.recv::<Passed>().await?)?;
            }
            9 => {
                Verification::decode(branch.recv::<Failed>().await?)?;
            }
            _ => return Err(ShellError::message("unexpected verification branch")),
        }
    };
    resource.send::<CandidateReady>(&checked.bytes()).await?;
    let branch = resource.offer().await?;
    match branch.label() {
        26 => {
            branch.recv::<CommitReady>().await?;
        }
        27 => {
            branch.recv::<CommitDenied>().await?;
            endpoint.send::<LoanR<'_>>(resource).await?;
            return Err(ShellError::message("candidate verification failed"));
        }
        _ => return Err(ShellError::message("unexpected commit branch")),
    }
    let mut second = endpoint.recv::<LoanS<'_>>().await?;
    let payload = checked.bytes();
    tokio::try_join!(
        resource.send::<CommitR>(&payload),
        second.send::<CommitS>(&payload)
    )?;
    let (first, second_result) =
        tokio::try_join!(resource.recv::<CommittedR>(), second.recv::<CommittedS>())?;
    if first != second_result {
        return Err(ShellError::message("resource results differ"));
    }
    let published = Artifact::decode(first);
    endpoint.send::<LoanS<'_>>(second).await?;
    endpoint.send::<LoanR<'_>>(resource).await?;
    endpoint.send::<Report>(&published.bytes()).await?;
    Ok(published)
}

pub async fn service(
    endpoint: &mut Endpoint<'_, STORE>,
    repository: &mut Repository<'_>,
    result: &VerificationInput,
) -> ShellResult<()> {
    let (checked, independent) = {
        let [mut candidate, mut independent] = endpoint.par()?;
        tokio::try_join!(
            async {
                let proposal = Artifact::decode(candidate.recv::<Prepare>().await?);
                let artifact = repository.prepare(proposal)?;
                candidate.send::<Prepared>(&artifact.bytes()).await?;
                let requested = Artifact::decode(candidate.recv::<Inspect>().await?);
                let checked = repository.verify(requested)?;
                result.record(checked);
                candidate.send::<Inspected>(&checked.bytes()).await?;
                loop {
                    let branch = candidate.offer().await?;
                    match branch.label() {
                        5 => {
                            let proposal = Artifact::decode(branch.recv::<Revise>().await?);
                            let artifact = repository.prepare(proposal)?;
                            candidate.send::<Revised>(&artifact.bytes()).await?;
                            let requested = Artifact::decode(candidate.recv::<Reinspect>().await?);
                            let checked = repository.verify(requested)?;
                            result.record(checked);
                            let branch = candidate.offer().await?;
                            match branch.label() {
                                8 => {
                                    branch.send::<Passed>(&checked.bytes()).await?;
                                }
                                9 => {
                                    branch.send::<Failed>(&checked.bytes()).await?;
                                }
                                _ => {
                                    return Err(ShellError::message(
                                        "unexpected verifier decision",
                                    ));
                                }
                            }
                        }
                        25 => {
                            let declared =
                                Verification::decode(branch.recv::<CandidateReady>().await?)?;
                            let actual = result.actual()?;
                            if declared != actual {
                                return Err(ShellError::message(
                                    "final verification differs from the actual result",
                                ));
                            }
                            break Ok(actual);
                        }
                        _ => return Err(ShellError::message("unexpected repository request")),
                    }
                }
            },
            async { Ok(independent.recv::<WorkReport>().await?) }
        )?
    };
    let branch = endpoint.offer().await?;
    match branch.label() {
        26 => {
            branch.send::<CommitReady>(&checked.bytes()).await?;
            endpoint.send::<CommitReady>(&checked.bytes()).await?;
            endpoint.send::<CommitReady>(&checked.bytes()).await?;
            let (first, second) = {
                let [mut first, mut second] = endpoint.par()?;
                tokio::try_join!(first.recv::<CommitR>(), second.recv::<CommitS>())?
            };
            let first = Verification::decode(first)?;
            let second = Verification::decode(second)?;
            if first != checked
                || second != checked
                || !checked.passed
                || checked.artifact != repository.current
            {
                return Err(ShellError::message(
                    "commit differs from the actual verified artifact",
                ));
            }
            repository.published.push((checked.artifact, independent));
            let mut observations = repository.observations.take();
            observations.push(checked.artifact);
            repository.observations.set(observations);
            {
                let bytes = checked.artifact.bytes();
                let [mut first, mut second] = endpoint.par()?;
                tokio::try_join!(
                    first.send::<CommittedR>(&bytes),
                    second.send::<CommittedS>(&bytes)
                )?;
            }
            let target = endpoint.recv::<Audit>().await?;
            let (published, _) = repository
                .published
                .last()
                .ok_or_else(|| ShellError::message("no published artifact"))?;
            if target != published.target {
                return Err(ShellError::message("audit target differs"));
            }
            endpoint.send::<Audited>(&published.bytes()).await?;
            Ok(())
        }
        27 => {
            branch.send::<CommitDenied>(&checked.bytes()).await?;
            endpoint.send::<CommitDenied>(&checked.bytes()).await?;
            endpoint.send::<CommitDenied>(&checked.bytes()).await?;
            Ok(())
        }
        _ => Err(ShellError::message("unexpected final repository decision")),
    }
}

pub async fn independent(endpoint: &mut Endpoint<'_, C>, input: u64) -> ShellResult<u64> {
    let mut resource = endpoint.recv::<LoanS<'_>>().await?;
    resource.send::<Work>(&input).await?;
    let value = resource.recv::<Worked>().await?;
    endpoint.send::<LoanS<'_>>(resource).await?;
    endpoint.send::<WorkReport>(&value).await?;
    Ok(value)
}

pub async fn auxiliary(endpoint: &mut Endpoint<'_, AUX>) -> ShellResult<()> {
    let value = endpoint.recv::<Work>().await?;
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    endpoint.send::<Worked>(&value.wrapping_mul(value)).await?;
    Ok(())
}

pub async fn second_owner<'r>(
    endpoint: &mut Endpoint<'r, S_OWNER>,
    resource: Endpoint<'r, S>,
) -> ShellResult<()> {
    endpoint.send::<LoanS<'_>>(resource).await?;
    let resource = endpoint.recv::<LoanS<'_>>().await?;
    let branch = endpoint.offer().await?;
    match branch.label() {
        26 => {
            branch.recv::<CommitReady>().await?;
            endpoint.send::<LoanS<'_>>(resource).await?;
            let _resource = endpoint.recv::<LoanS<'_>>().await?;
            Ok(())
        }
        27 => {
            branch.recv::<CommitDenied>().await?;
            Ok(())
        }
        _ => Err(ShellError::message("unexpected second-resource request")),
    }
}

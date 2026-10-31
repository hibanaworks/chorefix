//! Artifact bytes and actual computation results; no protocol progress is stored here.
use crate::shell::{ShellError, ShellResult};
use hibana::runtime::resolver::{DecisionArm, ResolverError};
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Artifact {
    pub target: u64,
    pub revision: u64,
    pub value: u64,
}
impl Artifact {
    pub fn bytes(self) -> [u8; 24] {
        let mut bytes = [0; 24];
        for (word, output) in [self.target, self.revision, self.value]
            .into_iter()
            .zip(bytes.as_chunks_mut::<8>().0)
        {
            output.copy_from_slice(&word.to_be_bytes());
        }
        bytes
    }
    pub fn decode(bytes: [u8; 24]) -> Self {
        Self {
            target: u64::from_be_bytes(bytes[..8].try_into().unwrap()),
            revision: u64::from_be_bytes(bytes[8..16].try_into().unwrap()),
            value: u64::from_be_bytes(bytes[16..].try_into().unwrap()),
        }
    }
}

/// The verifier's actual result for one exact artifact snapshot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Verification {
    pub artifact: Artifact,
    pub passed: bool,
}
impl Verification {
    pub fn bytes(self) -> [u8; 25] {
        let mut bytes = [0; 25];
        bytes[..24].copy_from_slice(&self.artifact.bytes());
        bytes[24] = u8::from(self.passed);
        bytes
    }
    pub fn decode(bytes: [u8; 25]) -> ShellResult<Self> {
        if bytes[24] > 1 {
            return Err(ShellError::message("invalid verification result"));
        }
        Ok(Self {
            artifact: Artifact::decode(bytes[..24].try_into().unwrap()),
            passed: bytes[24] == 1,
        })
    }
}

pub struct Repository<'a> {
    pub current: Artifact,
    pub published: Vec<(Artifact, u64)>,
    /// Append-only test observations, never consulted to admit an operation.
    pub observations: &'a Cell<Vec<Artifact>>,
}
impl Repository<'_> {
    pub fn prepare(&mut self, proposal: Artifact) -> ShellResult<Artifact> {
        if proposal.target != self.current.target || proposal.revision != self.current.revision {
            return Err(ShellError::message(
                "proposal target or base revision differs",
            ));
        }
        self.current = Artifact {
            revision: proposal
                .revision
                .checked_add(1)
                .ok_or_else(|| ShellError::message("artifact revision overflow"))?,
            ..proposal
        };
        Ok(self.current)
    }
    pub fn verify(&self, requested: Artifact) -> ShellResult<Verification> {
        if requested != self.current {
            return Err(ShellError::message(
                "verification target or revision differs",
            ));
        }
        Ok(Verification {
            artifact: requested,
            passed: requested.value.is_multiple_of(2),
        })
    }
}

/// The resolver observes a real verification result, not a workflow flag.
#[derive(Default)]
pub struct VerificationInput(Cell<Option<Verification>>);
impl VerificationInput {
    pub fn record(&self, result: Verification) {
        self.0.set(Some(result));
    }
    pub fn actual(&self) -> ShellResult<Verification> {
        self.0
            .get()
            .ok_or_else(|| ShellError::message("missing actual verification result"))
    }
    pub fn decide(&self) -> Result<DecisionArm, ResolverError> {
        match self.0.get() {
            Some(result) => Ok(if result.passed {
                DecisionArm::Left
            } else {
                DecisionArm::Right
            }),
            None => Err(ResolverError::reject()),
        }
    }
}

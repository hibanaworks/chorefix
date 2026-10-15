#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Work {
    Think,
    Eat,
}

pub trait Domain {
    type Error;
    type Call: WorkRequest;

    fn request(&self, agent: &'static str)
    -> impl Future<Output = Result<Self::Call, Self::Error>>;

    fn execute(
        &self,
        agent: &'static str,
        call: Self::Call,
    ) -> impl Future<Output = Result<(), Self::Error>>;
}

pub trait WorkRequest {
    fn work(&self) -> Work;
}

impl WorkRequest for Work {
    fn work(&self) -> Work {
        *self
    }
}

#[derive(Debug)]
pub enum RunError<E> {
    Attach(hibana::runtime::AttachError),
    Endpoint(hibana::EndpointError),
    Domain(E),
}

impl<E> From<hibana::EndpointError> for RunError<E> {
    fn from(error: hibana::EndpointError) -> Self {
        Self::Endpoint(error)
    }
}

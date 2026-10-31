#[derive(Debug)]
pub enum RunError<E> {
    Attach(hibana::runtime::AttachError),
    Endpoint(hibana::EndpointError),
    Domain(E),
    Resolver(hibana::runtime::resolver::ResolverError),
}

impl<E> From<hibana::EndpointError> for RunError<E> {
    fn from(error: hibana::EndpointError) -> Self {
        Self::Endpoint(error)
    }
}

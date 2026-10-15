use core::fmt;

use hibana::runtime::{AttachError, wire::CodecError};
use hibana_wasip1_runtime::{WasmError, exchange::ExchangeError};

pub type ShellResult<T> = Result<T, ShellError>;

#[derive(Debug)]
pub enum ShellError {
    Message(String),
    Io(std::io::Error),
    Attach(AttachError),
    Endpoint(hibana::EndpointError),
    Codec(CodecError),
    Wasm(WasmError),
    Exchange(ExchangeError),
}

impl ShellError {
    pub fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Message(message) => formatter.write_str(message),
            Self::Io(error) => write!(formatter, "{error}"),
            Self::Attach(error) => write!(formatter, "{error:?}"),
            Self::Endpoint(error) => write!(formatter, "{error:?}"),
            Self::Codec(error) => write!(formatter, "{error:?}"),
            Self::Wasm(error) => write!(formatter, "{error:?}"),
            Self::Exchange(error) => write!(formatter, "{error:?}"),
        }
    }
}

impl From<std::io::Error> for ShellError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<AttachError> for ShellError {
    fn from(error: AttachError) -> Self {
        Self::Attach(error)
    }
}

impl From<hibana::EndpointError> for ShellError {
    fn from(error: hibana::EndpointError) -> Self {
        Self::Endpoint(error)
    }
}

impl From<CodecError> for ShellError {
    fn from(error: CodecError) -> Self {
        Self::Codec(error)
    }
}

impl From<WasmError> for ShellError {
    fn from(error: WasmError) -> Self {
        Self::Wasm(error)
    }
}

impl From<ExchangeError> for ShellError {
    fn from(error: ExchangeError) -> Self {
        Self::Exchange(error)
    }
}

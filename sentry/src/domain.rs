use std::error;
use std::fmt;
use std::pin::Pin;

use futures::Future;

pub use self::asset::Asset;
pub use self::bignum::BigNum;
pub use self::channel::{Channel, ChannelListParams, ChannelRepository, ChannelSpec};
pub use self::validator::ValidatorDesc;

pub mod bignum;
pub mod channel;
pub mod validator;
pub mod asset;

#[cfg(test)]
/// re-exports all the fixtures in one module
pub mod fixtures {
    pub use super::asset::fixtures::*;
    pub use super::channel::fixtures::*;
    pub use super::validator::fixtures::*;
}

#[derive(Debug)]
pub enum DomainError {
    InvalidArgument(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Domain error", )
    }
}

impl error::Error for DomainError {
    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

pub trait IOError: error::Error + Send {}

#[derive(Debug)]
pub enum RepositoryError {
    /// An error with the underlying implementation occurred
    IO(Box<dyn IOError>),
    /// Error handling save errors, like Primary key already exists and etc.
    /// @TODO: Add and underlying implementation for this error
    User,
}

pub type RepositoryFuture<T> = Pin<Box<Future<Output=Result<T, RepositoryError>> + Send>>;

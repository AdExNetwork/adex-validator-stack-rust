use std::pin::Pin;

use futures::{Future, FutureExt};

use domain::validator::message::State;
use domain::{Asset, BigNum, Channel};

use crate::sanity::SanityChecker;

pub type AdapterFuture<T> = Pin<Box<dyn Future<Output = Result<T, AdapterError>> + Send>>;

#[derive(Debug, Eq, PartialEq)]
pub enum AdapterError {
    Authentication(String),
}

pub trait Adapter: SanityChecker + State {
    fn config(&self) -> &Config;

    fn validate_channel(&self, channel: &Channel) -> AdapterFuture<bool> {
        futures::future::ok(Self::check(&self.config(), &channel).is_ok()).boxed()
    }

    /// Signs the provided state_root
    fn sign(&self, state_root: &str) -> AdapterFuture<String>;

    /// Verify, based on the signature & state_root, that the signer is the same
    fn verify(
        &self,
        signer: &str,
        state_root: Self::StateRoot,
        signature: Self::Signature,
    ) -> AdapterFuture<bool>;

    /// Gets authentication for specific validator
    fn get_auth(&self, validator: &str) -> AdapterFuture<String>;
}

pub struct Config {
    pub identity: String,
    pub validators_whitelist: Vec<String>,
    pub creators_whitelist: Vec<String>,
    pub assets_whitelist: Vec<Asset>,
    pub minimal_deposit: BigNum,
    pub minimal_fee: BigNum,
}

pub struct ConfigBuilder {
    identity: String,
    validators_whitelist: Vec<String>,
    creators_whitelist: Vec<String>,
    assets_whitelist: Vec<Asset>,
    minimal_deposit: BigNum,
    minimal_fee: BigNum,
}

impl ConfigBuilder {
    pub fn new(identity: &str) -> Self {
        Self {
            identity: identity.to_string(),
            validators_whitelist: Vec::new(),
            creators_whitelist: Vec::new(),
            assets_whitelist: Vec::new(),
            minimal_deposit: 0.into(),
            minimal_fee: 0.into(),
        }
    }

    pub fn set_validators_whitelist(mut self, validators: &[&str]) -> Self {
        self.validators_whitelist = validators.iter().map(|slice| slice.to_string()).collect();
        self
    }

    pub fn set_creators_whitelist(mut self, creators: &[&str]) -> Self {
        self.creators_whitelist = creators.iter().map(|slice| slice.to_string()).collect();
        self
    }

    pub fn set_assets_whitelist(mut self, assets: &[Asset]) -> Self {
        self.assets_whitelist = assets.to_vec();
        self
    }

    pub fn set_minimum_deposit(mut self, minimum: BigNum) -> Self {
        self.minimal_deposit = minimum;
        self
    }

    pub fn set_minimum_fee(mut self, minimum: BigNum) -> Self {
        self.minimal_fee = minimum;
        self
    }

    pub fn build(self) -> Config {
        Config {
            identity: self.identity,
            validators_whitelist: self.validators_whitelist,
            creators_whitelist: self.creators_whitelist,
            assets_whitelist: self.assets_whitelist,
            minimal_deposit: self.minimal_deposit,
            minimal_fee: self.minimal_fee,
        }
    }
}

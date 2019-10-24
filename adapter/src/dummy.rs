use primitives::adapter::{Adapter, AdapterError, AdapterOptions, AdapterResult, Session};
use primitives::channel_validator::ChannelValidator;
use primitives::config::Config;
use primitives::{Channel, ValidatorId};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct DummyAdapter {
    identity: ValidatorId,
    config: Config,
    // Auth tokens that we have verified (tokenId => session)
    session_tokens: HashMap<String, ValidatorId>,
    // Auth tokens that we've generated to authenticate with someone (address => token)
    authorization_tokens: HashMap<String, String>,
}

// Enables DummyAdapter to be able to
// check if a channel is valid
impl ChannelValidator for DummyAdapter {}

impl Adapter for DummyAdapter {
    type Output = DummyAdapter;

    fn init(opts: AdapterOptions, config: &Config) -> AdapterResult<DummyAdapter> {
        let (identity, session_tokens, authorization_tokens) = match opts {
            AdapterOptions::DummAdapter {
                dummy_identity,
                dummy_auth,
                dummy_auth_tokens,
            } => (dummy_identity, dummy_auth, dummy_auth_tokens),
            _ => {
                return Err(AdapterError::Configuration(
                    "dummy_identity, dummy_auth, dummy_auth_tokens required".to_string(),
                ))
            }
        };

        Ok(Self {
            identity,
            config: config.to_owned(),
            session_tokens,
            authorization_tokens,
        })
    }

    fn unlock(&mut self) -> AdapterResult<()> {
        Ok(())
    }

    fn whoami(&self) -> ValidatorId {
        self.identity.clone()
    }

    fn sign(&self, state_root: &str) -> AdapterResult<String> {
        let signature = format!(
            "Dummy adapter signature for {} by {}",
            state_root,
            self.whoami().to_hex_prefix_string()
        );
        Ok(signature)
    }

    fn verify(
        &self,
        signer: &ValidatorId,
        _state_root: &str,
        signature: &str,
    ) -> AdapterResult<bool> {
        // select the `identity` and compare it to the signer
        // for empty string this will return array with 1 element - an empty string `[""]`
        let is_same = match signature.rsplit(' ').take(1).next() {
            Some(from) => from == signer.to_hex_prefix_string(),
            None => false,
        };

        Ok(is_same)
    }

    fn validate_channel(&self, channel: &Channel) -> AdapterResult<bool> {
        match DummyAdapter::is_channel_valid(&self.config, channel) {
            Ok(_) => Ok(true),
            Err(e) => Err(AdapterError::InvalidChannel(e.to_string())),
        }
    }

    fn session_from_token(&mut self, token: &str) -> AdapterResult<Session> {
        let identity = self
            .authorization_tokens
            .iter()
            .find(|(_, id)| *id == token);

        match identity {
            Some((id, _)) => Ok(Session {
                uid: self.session_tokens[id].clone(),
                era: 0,
            }),
            None => Err(AdapterError::Authentication(format!(
                "no session token for this auth: {}",
                token
            ))),
        }
    }

    fn get_auth(&mut self, _validator: &ValidatorId) -> AdapterResult<String> {
        let who = self
            .session_tokens
            .iter()
            .find(|(_, id)| *id == &self.identity);
        match who {
            Some((id, _)) => {
                let auth = self.authorization_tokens.get(id).expect("id should exist");
                Ok(auth.to_owned())
            }
            None => Err(AdapterError::Authentication(format!(
                "no auth token for this identity: {}",
                self.identity
            ))),
        }
    }
}

use crate::channel::{Channel, ChannelError, SpecValidator, SpecValidators};
use crate::config::Config;
use crate::ValidatorId;
use chrono::Utc;
use std::cmp::PartialEq;

pub trait ChannelValidator {
    fn is_channel_valid(
        config: &Config,
        validator_identity: &ValidatorId,
        channel: &Channel,
    ) -> Result<(), ChannelError> {
        let adapter_channel_validator = match channel.spec.validators.find(validator_identity) {
            // check if the channel validators include our adapter identity
            SpecValidator::None => return Err(ChannelError::AdapterNotIncluded),
            SpecValidator::Leader(validator) | SpecValidator::Follower(validator) => validator,
        };

        if channel.valid_until < Utc::now() {
            return Err(ChannelError::PassedValidUntil);
        }

        if !all_validators_listed(&channel.spec.validators, &config.validators_whitelist) {
            return Err(ChannelError::UnlistedValidator);
        }

        if !creator_listed(&channel, &config.creators_whitelist) {
            return Err(ChannelError::UnlistedCreator);
        }

        if !asset_listed(&channel, &config.token_address_whitelist) {
            return Err(ChannelError::UnlistedAsset);
        }

        if channel.deposit_amount < config.minimal_deposit {
            return Err(ChannelError::MinimumDepositNotMet);
        }

        if adapter_channel_validator.fee < config.minimal_fee {
            return Err(ChannelError::MinimumValidatorFeeNotMet);
        }

        Ok(())
    }
}

pub fn all_validators_listed(validators: &SpecValidators, whitelist: &[ValidatorId]) -> bool {
    if whitelist.is_empty() {
        true
    } else {
        let found_validators = whitelist
            .iter()
            .filter(|&allowed| {
                allowed == &validators.leader().id || allowed == &validators.follower().id
            })
            // this will ensure that if we find the 2 validators earlier
            // we don't go over the other values of the whitelist
            .take(2);
        // the found validators should be exactly 2, if they are not, then 1 or 2 are missing
        found_validators.count() == 2
    }
}

pub fn creator_listed(channel: &Channel, whitelist: &[ValidatorId]) -> bool {
    // if the list is empty, return true, as we don't have a whitelist to restrict us to
    // or if we have a list, check if it includes the `channel.creator`
    whitelist.is_empty() || whitelist.iter().any(|allowed| allowed.eq(&channel.creator))
}

pub fn asset_listed(channel: &Channel, whitelist: &[String]) -> bool {
    // if the list is empty, return true, as we don't have a whitelist to restrict us to
    // or if we have a list, check if it includes the `channel.deposit_asset`
    whitelist.is_empty()
        || whitelist
            .iter()
            .any(|allowed| allowed == &channel.deposit_asset)
}

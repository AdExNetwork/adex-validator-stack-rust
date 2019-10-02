use num::rational::Ratio;
use num_traits::CheckedSub;
use primitives::{BalancesMap, BigNum, Channel, DomainError, ValidatorDesc};

pub fn get_balances_after_fees_tree(
    balances: &BalancesMap,
    channel: &Channel,
) -> Result<BalancesMap, DomainError> {
    let deposit_amount = channel.deposit_amount.clone();

    let total_distributed = balances.iter().map(|(_, balance)| balance).sum::<BigNum>();

    let validators_iter = channel.spec.validators.into_iter();
    let total_validators_fee = validators_iter
        .map(|validator| &validator.fee)
        .sum::<BigNum>();

    if total_validators_fee > deposit_amount {
        return Err(DomainError::RuleViolation(
            "total fees <= deposit: fee constraint violated".into(),
        ));
    }

    if total_distributed > deposit_amount {
        return Err(DomainError::RuleViolation(
            "distributed <= deposit: OUTPACE rule #4".into(),
        ));
    }

    let deposit_to_distribute = &deposit_amount - &total_validators_fee;

    let ratio = Ratio::new(deposit_to_distribute.clone(), deposit_amount.clone());
    let fee_ratio = Ratio::new(total_distributed.clone(), deposit_amount.clone());

    let mut balances_after_fees = BalancesMap::default();
    let mut total = BigNum::from(0);

    for (key, value) in balances.iter() {
        let adjusted_balance = value * &ratio;

        total += &adjusted_balance;
        balances_after_fees.insert(key.clone(), adjusted_balance);
    }

    let rounding_error = if deposit_amount == total_distributed {
        deposit_to_distribute.checked_sub(&total).ok_or_else(|| {
            DomainError::RuleViolation("rounding_err should never be negative".to_owned())
        })?
    } else {
        BigNum::from(0)
    };

    let balances_after_fees = distribute_fee(
        balances_after_fees,
        rounding_error,
        fee_ratio,
        channel.spec.validators.into_iter(),
    );

    Ok(balances_after_fees)
}

fn distribute_fee<'a>(
    mut balances: BalancesMap,
    rounding_error: BigNum,
    fee_ratio: Ratio<BigNum>,
    validators: impl Iterator<Item = &'a ValidatorDesc>,
) -> BalancesMap {
    for (index, validator) in validators.enumerate() {
        let fee = &validator.fee * &fee_ratio;

        let fee_rounded = if index == 0 {
            &fee + &rounding_error
        } else {
            fee
        };

        if fee_rounded > 0.into() {
            let entry = balances
                .entry(validator.id.clone())
                .or_insert_with(|| 0.into());

            *entry += &fee_rounded;
        }
    }

    balances
}

#[cfg(test)]
mod test {
    use super::*;
    use primitives::util::tests::prep_db::{
        DUMMY_CHANNEL, DUMMY_VALIDATOR_FOLLOWER, DUMMY_VALIDATOR_LEADER, IDS,
    };

    mod applying_fee_returns_the_same_tree_with_zero_fees {
        use super::*;
        fn setup_balances_map(balances_map: &BalancesMap) -> BalancesMap {
            let channel = get_zero_fee_channel();

            get_balances_after_fees_tree(balances_map, &channel)
                .expect("Calculation of fees failed")
        }

        #[test]
        fn case_1_three_values() {
            let balances_map: BalancesMap = vec![
                ("a".to_string(), 1001.into()),
                ("b".to_string(), 3124.into()),
                ("c".to_string(), 122.into()),
            ]
            .into_iter()
            .collect();

            assert_eq!(setup_balances_map(&balances_map), balances_map);
        }

        #[test]
        fn case_2_three_simple_values() {
            let balances_map: BalancesMap = vec![
                ("a".to_string(), 1.into()),
                ("b".to_string(), 2.into()),
                ("c".to_string(), 3.into()),
            ]
            .into_iter()
            .collect();

            assert_eq!(setup_balances_map(&balances_map), balances_map);
        }

        #[test]
        fn case_3_one_value() {
            let balances_map = vec![("a".to_string(), BigNum::from(1))]
                .into_iter()
                .collect();

            assert_eq!(setup_balances_map(&balances_map), balances_map);
        }

        #[test]
        fn case_4_two_values() {
            let balances_map = vec![
                ("a".to_string(), 1.into()),
                ("b".to_string(), 99_999.into()),
            ]
            .into_iter()
            .collect();

            assert_eq!(setup_balances_map(&balances_map), balances_map);
        }

        fn get_zero_fee_channel() -> Channel {
            let leader = ValidatorDesc {
                fee: 0.into(),
                ..DUMMY_VALIDATOR_LEADER.clone()
            };
            let follower = ValidatorDesc {
                fee: 0.into(),
                ..DUMMY_VALIDATOR_FOLLOWER.clone()
            };

            let mut spec = DUMMY_CHANNEL.spec.clone();
            spec.validators = [leader, follower].into();

            Channel {
                deposit_amount: 100_000.into(),
                spec,
                ..DUMMY_CHANNEL.clone()
            }
        }
    }

    mod applying_fee_correctly {
        use super::*;

        fn setup_balances_after_fee(balances_map: BalancesMap) -> BalancesMap {
            let leader = ValidatorDesc {
                fee: 50.into(),
                ..DUMMY_VALIDATOR_LEADER.clone()
            };
            let follower = ValidatorDesc {
                fee: 50.into(),
                ..DUMMY_VALIDATOR_FOLLOWER.clone()
            };

            let mut spec = DUMMY_CHANNEL.spec.clone();
            spec.validators = [leader, follower].into();

            let channel = Channel {
                deposit_amount: 10_000.into(),
                spec,
                ..DUMMY_CHANNEL.clone()
            };

            get_balances_after_fees_tree(&balances_map, &channel)
                .expect("Calculation of fees failed")
        }

        #[test]
        fn case_1_partially_distributed() {
            let balances_map = vec![
                ("a".to_string(), 1_000.into()),
                ("b".to_string(), 1_200.into()),
            ]
            .into_iter()
            .collect();

            let expected_balances: BalancesMap = vec![
                ("a".to_string(), 990.into()),
                ("b".to_string(), 1_188.into()),
                (IDS.get("leader").unwrap().to_owned(), 11.into()),
                (IDS.get("follower").unwrap().to_owned(), 11.into()),
            ]
            .into_iter()
            .collect();

            let balances_after_fee = setup_balances_after_fee(balances_map);
            let actual_sum: BigNum = balances_after_fee.iter().map(|(_, v)| v).sum();

            assert_eq!(
                expected_balances
                    .iter()
                    .map(|(_, value)| value)
                    .sum::<BigNum>(),
                actual_sum
            );
            assert_eq!(expected_balances, balances_after_fee);
        }

        #[test]
        fn case_2_partially_distributed_with_validator_in_the_input_balances_map() {
            let balances_map = vec![
                ("a".to_string(), 100.into()),
                ("b".to_string(), 2_000.into()),
                (IDS.get("leader").unwrap().to_owned(), 200.into()),
            ]
            .into_iter()
            .collect();

            let expected_balances: BalancesMap = vec![
                ("a".to_string(), 99.into()),
                ("b".to_string(), 1_980.into()),
                (IDS.get("leader").unwrap().to_owned(), 209.into()),
                (IDS.get("follower").unwrap().to_owned(), 11.into()),
            ]
            .into_iter()
            .collect();

            let balances_after_fee = setup_balances_after_fee(balances_map);
            let actual_sum: BigNum = balances_after_fee.iter().map(|(_, v)| v).sum();

            assert_eq!(
                expected_balances
                    .iter()
                    .map(|(_, value)| value)
                    .sum::<BigNum>(),
                actual_sum
            );
            assert_eq!(expected_balances, balances_after_fee);
        }

        #[test]
        /// also testing the rounding error correction
        fn case_3_fully_distributed() {
            let balances_map = vec![
                ("a".to_string(), 105.into()),
                ("b".to_string(), 195.into()),
                ("c".to_string(), 700.into()),
                ("d".to_string(), 5_000.into()),
                ("e".to_string(), 4_000.into()),
            ]
            .into_iter()
            .collect();

            let expected_balances: BalancesMap = vec![
                ("a".to_string(), 103.into()),
                ("b".to_string(), 193.into()),
                ("c".to_string(), 693.into()),
                ("d".to_string(), 4_950.into()),
                ("e".to_string(), 3_960.into()),
                (IDS.get("leader").unwrap().to_owned(), 51.into()),
                (IDS.get("follower").unwrap().to_owned(), 50.into()),
            ]
            .into_iter()
            .collect();

            let balances_after_fee = setup_balances_after_fee(balances_map);
            let actual_sum: BigNum = balances_after_fee.iter().map(|(_, v)| v).sum();

            assert_eq!(
                expected_balances
                    .iter()
                    .map(|(_, value)| value)
                    .sum::<BigNum>(),
                actual_sum
            );
            assert_eq!(expected_balances, balances_after_fee);
        }
    }

    #[test]
    fn errors_when_fees_larger_that_deposit() {
        let balances_map = vec![("a".to_string(), 10.into()), ("b".to_string(), 10.into())]
            .into_iter()
            .collect();

        let leader = ValidatorDesc {
            fee: 600.into(),
            ..DUMMY_VALIDATOR_LEADER.clone()
        };
        let follower = ValidatorDesc {
            fee: 600.into(),
            ..DUMMY_VALIDATOR_FOLLOWER.clone()
        };

        let mut spec = DUMMY_CHANNEL.spec.clone();
        spec.validators = [leader, follower].into();

        let channel = Channel {
            deposit_amount: 1_000.into(),
            spec,
            ..DUMMY_CHANNEL.clone()
        };

        let domain_error = get_balances_after_fees_tree(&balances_map, &channel)
            .expect_err("Should be DomainError not allow fees sum to exceed the deposit");

        assert_eq!(
            DomainError::RuleViolation(
                "total fees <= deposit: fee constraint violated".to_string()
            ),
            domain_error
        );
    }
}

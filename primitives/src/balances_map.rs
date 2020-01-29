use std::collections::BTreeMap;

use crate::{BigNum, ValidatorId};
use std::collections::btree_map::{Entry, Iter, Values};

use serde::{Deserialize, Serialize};
use std::iter::FromIterator;
use std::ops::Index;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(transparent)]
pub struct BalancesMap(BTreeMap<ValidatorId, BigNum>);

impl Index<&'_ ValidatorId> for BalancesMap {
    type Output = BigNum;

    fn index(&self, index: &ValidatorId) -> &Self::Output {
        self.0.index(index)
    }
}

impl BalancesMap {
    pub fn iter(&self) -> Iter<'_, ValidatorId, BigNum> {
        self.0.iter()
    }

    pub fn values(&self) -> Values<'_, ValidatorId, BigNum> {
        self.0.values()
    }

    pub fn get(&self, key: &ValidatorId) -> Option<&BigNum> {
        self.0.get(key)
    }

    pub fn entry(&mut self, key: ValidatorId) -> Entry<'_, ValidatorId, BigNum> {
        self.0.entry(key)
    }

    pub fn insert(&mut self, key: ValidatorId, value: BigNum) -> Option<BigNum> {
        self.0.insert(key, value)
    }
}

impl FromIterator<(ValidatorId, BigNum)> for BalancesMap {
    fn from_iter<I: IntoIterator<Item = (ValidatorId, BigNum)>>(iter: I) -> Self {
        // @TODO: Is there better way to do this?
        let btree_map: BTreeMap<ValidatorId, BigNum> = iter.into_iter().collect();

        BalancesMap(btree_map)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::tests::prep_db::IDS;
    use crate::BigNum;

    #[test]
    fn test_balances_map_serialization() {
        let data = vec![
            (IDS["leader"].clone(), BigNum::from(50_u64)),
            (IDS["follower"].clone(), BigNum::from(100_u64)),
        ];

        let balances_map: BalancesMap = data.into_iter().collect();

        let actual_json = serde_json::to_string(&balances_map).expect("Should serialize it");
        // should be all lowercase!
        let expected_json = r#"{"0xC91763D7F14ac5c5dDfBCD012e0D2A61ab9bDED3":"100","0xce07CbB7e054514D590a0262C93070D838bFBA2e":"50"}"#;

        assert_eq!(expected_json, actual_json);

        let balances_map_from_json: BalancesMap =
            serde_json::from_str(&actual_json).expect("Should deserialize it");

        assert_eq!(balances_map, balances_map_from_json);
    }
}

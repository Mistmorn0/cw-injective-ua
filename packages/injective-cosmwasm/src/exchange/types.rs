use cosmwasm_std::{Empty, StdError, StdResult};
use cw_storage_plus::{Key, KeyDeserialize, Prefixer, PrimaryKey};
use injective_math::FPDecimal;
use schemars::JsonSchema;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::fmt;

use crate::InjectiveQuerier;

pub const UNSORTED_CANCELLATION_STRATEGY: i32 = 0;
pub const FROM_WORST_TO_BEST_CANCELLATION_STRATEGY: i32 = 1;

/// Deposit is data format for the subaccount deposit
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Deposit {
    pub available_balance: FPDecimal,
    pub total_balance: FPDecimal,
}

/// Response to query for aggregate volume for a given market
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct DenomDecimals {
    pub denom: String,
    pub decimals: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct PriceLevel {
    pub p: FPDecimal,
    pub q: FPDecimal,
}

impl PriceLevel {
    // helper method for tests
    pub fn new(p: FPDecimal, q: FPDecimal) -> PriceLevel {
        PriceLevel { p, q }
    }
}

/// Volume values divided by type (maker or taker volume)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct VolumeByType {
    pub maker_volume: FPDecimal,
    pub taker_volume: FPDecimal,
}

/// Total volume on a given market
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MarketVolume {
    pub market_id: MarketId,
    pub volume: VolumeByType,
}

pub enum MarketType {
    Spot,
    Derivative,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema)]
pub struct MarketId(String);

impl MarketId {
    pub fn new<S>(market_id_s: S) -> StdResult<Self>
    where
        S: Into<String>,
    {
        let market_id = market_id_s.into();

        if !market_id.starts_with("0x") {
            return Err(StdError::generic_err("Invalid prefix: market_id must start with 0x"));
        }

        if market_id.len() != 66 {
            return Err(StdError::generic_err("Invalid length: market_id must be exactly 66 characters"));
        }

        Ok(Self(market_id.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn validate(self, querier: &InjectiveQuerier, market_type: MarketType) -> StdResult<Self> {
        match market_type {
            MarketType::Spot => {
                let _spot_market = querier.query_spot_market(&self)?;
                Empty {}
            }
            MarketType::Derivative => {
                let _derivative_market = querier.query_derivative_market(&self)?;
                Empty {}
            }
        };

        Ok(self)
    }

    pub fn unchecked<S>(market_id_s: S) -> Self
    where
        S: Into<String>,
    {
        Self(market_id_s.into().to_lowercase())
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for MarketId {
    fn into(self) -> String {
        self.0
    }
}

impl<'a> From<&'a str> for MarketId {
    fn from(s: &'a str) -> Self {
        MarketId::new(s).unwrap()
    }
}

impl<'de> Deserialize<'de> for MarketId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let market_id = String::deserialize(deserializer)?;

        if !market_id.starts_with("0x") {
            let error_message = format!("Invalid prefix in deserialization: market_id must start with 0x, received {}", market_id);
            return Err(D::Error::custom(error_message));
        }

        if market_id.len() != 66 {
            let error_message = format!(
                "Invalid length in deserialization: market_id must be exactly 66 characters, received {}",
                market_id
            );
            return Err(D::Error::custom(error_message));
        }

        Ok(MarketId::unchecked(market_id))
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema)]
pub struct SubaccountId(String);

impl SubaccountId {
    pub fn new<S>(subaccount_id_s: S) -> std::result::Result<SubaccountId, cosmwasm_std::StdError>
    where
        S: Into<String>,
    {
        let subaccount_id = subaccount_id_s.into();

        if !subaccount_id.starts_with("0x") {
            return Err(StdError::generic_err("Invalid prefix: subaccount_id must start with 0x"));
        }

        if subaccount_id.len() != 66 {
            return Err(StdError::generic_err("Invalid length: subaccount_id must be exactly 66 characters"));
        }

        Ok(Self(subaccount_id.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn unchecked<S>(subaccount_id_s: S) -> Self
    where
        S: Into<String>,
    {
        Self(subaccount_id_s.into().to_lowercase())
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl<'de> Deserialize<'de> for SubaccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let subaccount_id = String::deserialize(deserializer)?;

        if !subaccount_id.starts_with("0x") {
            let error_message = format!(
                "Invalid prefix in deserialization: subaccount_id must start with 0x, received {}",
                subaccount_id
            );
            return Err(D::Error::custom(error_message));
        }

        if subaccount_id.len() != 66 {
            let error_message = format!(
                "Invalid length in deserialization: subaccount_id must be exactly 66 characters, received {}",
                subaccount_id
            );
            return Err(D::Error::custom(error_message));
        }

        Ok(SubaccountId::unchecked(subaccount_id))
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema)]
pub struct ShortSubaccountId(String);

const MAX_SHORT_SUBACCOUNT_NONCE: u16 = 999;

impl ShortSubaccountId {
    pub fn new<S>(id_s: S) -> Result<ShortSubaccountId, StdError>
    where
        S: Into<String>,
    {
        let id = id_s.into();

        match id.parse::<u16>() {
            Ok(value) if value <= MAX_SHORT_SUBACCOUNT_NONCE => Ok(Self(id)),
            _ => Err(StdError::generic_err("Invalid value: ShortSubaccountId must be a number between 0-999")),
        }
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn unchecked<S>(id_s: S) -> Self
    where
        S: Into<String>,
    {
        Self(id_s.into())
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl<'de> Deserialize<'de> for ShortSubaccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id = String::deserialize(deserializer)?;

        match id.parse::<u16>() {
            Ok(value) if value <= MAX_SHORT_SUBACCOUNT_NONCE => Ok(ShortSubaccountId::unchecked(id)),
            _ => Err(D::Error::custom(format!(
                "Invalid value in deserialization: ShortSubaccountId must be a number between 0-999, received {}",
                id
            ))),
        }
    }
}

impl From<SubaccountId> for ShortSubaccountId {
    fn from(subaccount_id: SubaccountId) -> Self {
        let last_three_chars = &subaccount_id.as_str()[subaccount_id.as_str().len() - 3..];
        ShortSubaccountId::unchecked(last_three_chars.to_string())
    }
}

impl fmt::Display for SubaccountId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for SubaccountId {
    fn into(self) -> String {
        self.0
    }
}

impl KeyDeserialize for SubaccountId {
    type Output = SubaccountId;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Ok(SubaccountId::unchecked(String::from_vec(value)?))
    }
}

impl<'a> PrimaryKey<'a> for SubaccountId {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        // this is simple, we don't add more prefixes
        vec![Key::Ref(self.as_bytes())]
    }
}

impl<'a> Prefixer<'a> for SubaccountId {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Ref(self.as_bytes())]
    }
}

impl KeyDeserialize for &SubaccountId {
    type Output = SubaccountId;

    #[inline(always)]
    fn from_vec(value: Vec<u8>) -> StdResult<Self::Output> {
        Self::Output::from_vec(value)
    }
}

impl AsRef<str> for SubaccountId {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> PrimaryKey<'a> for &'a SubaccountId {
    type Prefix = ();
    type SubPrefix = ();
    type Suffix = Self;
    type SuperSuffix = Self;

    fn key(&self) -> Vec<Key> {
        // this is simple, we don't add more prefixes
        vec![Key::Ref(self.as_ref().as_bytes())]
    }
}

impl<'a> Prefixer<'a> for &'a SubaccountId {
    fn prefix(&self) -> Vec<Key> {
        vec![Key::Ref(self.as_bytes())]
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, JsonSchema)]
pub struct Hash([u8; 32]);

impl Hash {
    pub fn new(bytes: [u8; 32]) -> Hash {
        Hash(bytes)
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        self.0
    }

    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    pub fn from_hex<T: AsRef<[u8]>>(s: T) -> StdResult<Hash> {
        let mut bytes = [0u8; 32];
        hex::decode_to_slice(s, &mut bytes).map_err(|e| StdError::generic_err(e.to_string()))?;
        Ok(Hash::new(bytes))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{}", self.to_hex())
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::StdError;

    use crate::{MarketId, SubaccountId};

    #[test]
    fn unchecked_subaccount_id_to_lowercase() {
        let subaccount_id = SubaccountId::unchecked("0xB5e09b93aCEb70C1711aF078922fA256011D7e56000000000000000000000045");
        let subaccount_id_str: String = subaccount_id.into();
        assert_eq!(
            subaccount_id_str,
            "0xB5e09b93aCEb70C1711aF078922fA256011D7e56000000000000000000000045".to_lowercase()
        );
    }

    #[test]
    fn unchecked_market_id_to_lowercase() {
        let market_id = MarketId::unchecked("0x01EDFAB47F124748DC89998EB33144AF734484BA07099014594321729A0CA16B");
        let market_id_str: String = market_id.into();
        assert_eq!(
            market_id_str,
            "0x01edfab47f124748dc89998eb33144af734484ba07099014594321729a0ca16b".to_lowercase()
        );
    }

    #[test]
    fn checked_subaccount_id_to_lowercase() {
        let subaccount_id = SubaccountId::new("0xB5e09b93aCEb70C1711aF078922fA256011D7e56000000000000000000000045").unwrap();
        let subaccount_id_str: String = subaccount_id.into();
        assert_eq!(
            subaccount_id_str,
            "0xB5e09b93aCEb70C1711aF078922fA256011D7e56000000000000000000000045".to_lowercase()
        );
    }

    #[test]
    fn checked_market_id_to_lowercase() {
        let market_id = MarketId::new("0x01EDFAB47F124748DC89998EB33144AF734484BA07099014594321729A0CA16B").unwrap();
        let market_id_str: String = market_id.into();
        assert_eq!(
            market_id_str,
            "0x01edfab47f124748dc89998eb33144af734484ba07099014594321729a0ca16b".to_lowercase()
        );
    }

    #[test]
    fn subaccount_id_checks() {
        let wrong_prefix_err = SubaccountId::new("00B5e09b93aCEb70C1711aF078922fA256011D7e56000000000000000000000045").unwrap_err();
        assert_eq!(
            wrong_prefix_err,
            StdError::generic_err("Invalid prefix: subaccount_id must start with 0x")
        );

        let wrong_length_err = SubaccountId::new("0xB5e09b93aCEb70C1711aF078922fA256011D7e5600000000000000000000004").unwrap_err();
        assert_eq!(
            wrong_length_err,
            StdError::generic_err("Invalid length: subaccount_id must be exactly 66 characters")
        );

        let wrong_length_err = SubaccountId::new("0xB5e09b93aCEb70C1711aF078922fA256011D7e560000000000000000000000451").unwrap_err();
        assert_eq!(
            wrong_length_err,
            StdError::generic_err("Invalid length: subaccount_id must be exactly 66 characters")
        );
    }

    #[test]
    fn market_id_checks() {
        let wrong_prefix_err = MarketId::new("0001EDFAB47F124748DC89998EB33144AF734484BA07099014594321729A0CA16B").unwrap_err();
        assert_eq!(wrong_prefix_err, StdError::generic_err("Invalid prefix: market_id must start with 0x"));

        let wrong_length_err = MarketId::new("0x01EDFAB47F124748DC89998EB33144AF734484BA07099014594321729A0CA16").unwrap_err();
        assert_eq!(
            wrong_length_err,
            StdError::generic_err("Invalid length: market_id must be exactly 66 characters")
        );

        let wrong_length_err = MarketId::new("0x01EDFAB47F124748DC89998EB33144AF734484BA07099014594321729A0CA16B2").unwrap_err();
        assert_eq!(
            wrong_length_err,
            StdError::generic_err("Invalid length: market_id must be exactly 66 characters")
        );
    }

    #[test]
    fn subaccount_id_unchecked_works() {
        let a = SubaccountId::unchecked("123");
        let aa = SubaccountId::unchecked(String::from("123"));
        let b = SubaccountId::unchecked("be");
        assert_eq!(a, aa);
        assert_ne!(a, b);
    }

    #[test]
    fn subaccount_id_as_str_works() {
        let subaccount_id = SubaccountId::unchecked("amazing-id");
        assert_eq!(subaccount_id.as_str(), "amazing-id");
    }

    #[test]
    fn subaccount_id_as_bytes_works() {
        let subaccount_id = SubaccountId::unchecked("literal-string");
        assert_eq!(
            subaccount_id.as_bytes(),
            [108, 105, 116, 101, 114, 97, 108, 45, 115, 116, 114, 105, 110, 103]
        );
    }

    #[test]
    fn subaccount_id_implements_as_ref_for_str() {
        let subaccount_id = SubaccountId::unchecked("literal-string");
        assert_eq!(subaccount_id.as_ref(), "literal-string");
    }

    #[test]
    fn subaccount_id_implements_display() {
        let subaccount_id = SubaccountId::unchecked("literal-string");
        assert_eq!(format!("{}", subaccount_id), "literal-string");
    }
}

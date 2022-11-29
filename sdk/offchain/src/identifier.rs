// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::subaddress::{Subaddress, SubaddressParseError};
use bech32::{self, u5, FromBase32, ToBase32};
use diem_sdk::{
    move_types::account_address::AccountAddressParseError,
    transaction_builder::Currency,
    types::{
        account_address::AccountAddress,
        account_config::{allowed_currency_code_string, XDX_NAME, XUS_NAME},
    },
};
use std::{collections::HashMap, str::FromStr};
use thiserror::Error;
use url::Url;

const DIEM_BECH32_VERSION_LENGTH: usize = 1;
const DIEM_BECH32_VERSION: u8 = 1;

/// Intent is a struct holdind data decoded from Diem Intent Identifier string
/// https://dip.diem.com/dip-5/#format
#[derive(Ord, PartialOrd, Eq, PartialEq, Hash, Clone)]
pub struct Intent {
    account_address: AccountAddress,
    subaddress: Subaddress,
    currency: Option<Currency>,
    amount: Option<u64>,
    hrp: HumanReadablePrefix,
}

impl Intent {
    /// Encode Intent as a Diem intent identifier (https://dip.diem.com/dip-5/).
    ///
    /// ## Example
    /// ```
    /// use offchain::identifier::Intent;
    /// use std::str::FromStr;
    /// let identifier = "diem://dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk?c=XUS&am=4500";
    /// let intent = Intent::from_str(identifier).unwrap();
    /// assert_eq!(intent.to_encoded_string().unwrap(), identifier);
    /// ```
    pub fn to_encoded_string(&self) -> Result<String, IntentIdentifierError> {
        let encoded_account = match encode_account(self.hrp, self.account_address, self.subaddress)
        {
            Ok(encoded_account) => encoded_account,
            Err(error) => return Err(IntentIdentifierError::Bech32(error)),
        };
        Ok(encode_intent(&encoded_account, self.currency, self.amount))
    }

    pub fn account_address(&self) -> &AccountAddress {
        &self.account_address
    }

    pub fn subaddress(&self) -> &Subaddress {
        &self.subaddress
    }

    pub fn currency(&self) -> Option<Currency> {
        self.currency
    }

    pub fn amount(&self) -> Option<u64> {
        self.amount
    }

    pub fn hrp(&self) -> &HumanReadablePrefix {
        &self.hrp
    }
}

impl FromStr for Intent {
    type Err = IntentIdentifierError;

    /// Decode Diem intent identifier (https://dip.diem.com/dip-5/) int 3 parts:
    /// 1. account identifier: account address & sub-address
    /// 2. currency
    /// 3. amount
    ///
    /// ## Example
    /// ```
    /// use offchain::identifier::HumanReadablePrefix;
    /// use offchain::identifier::Intent;
    /// use std::str::FromStr;
    /// let identifier = "diem://dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk?c=XUS&am=4500";
    /// let intent = Intent::from_str(identifier).unwrap();
    /// assert_eq!(intent.hrp(), &HumanReadablePrefix::DM);
    /// assert_eq!(intent.amount(), Some(4500));
    /// ```
    fn from_str(encoded_intent_identifier: &str) -> Result<Intent, IntentIdentifierError> {
        let result: Url = Url::parse(encoded_intent_identifier)?;
        if result.scheme() != "diem" {
            return Err(IntentIdentifierError::Parse(format!(
                "Unknown intent identifier scheme {}",
                result.scheme()
            )));
        }

        if result.host_str().is_none() {
            return Err(IntentIdentifierError::Parse(
                "No host string in encoded identifier".to_string(),
            ));
        }
        let (hrp, account_address, subaddress) = decode_account(result.host_str().unwrap())?;
        let params: HashMap<String, String> = result.query_pairs().into_owned().collect();
        let amount = normalize_amount(params.get("am"))?;
        let currency = normalize_currency(params.get("c"))?;

        Ok(Self {
            account_address,
            subaddress,
            currency,
            amount,
            hrp,
        })
    }
}

#[derive(Error, Debug)]
pub enum IntentIdentifierError {
    #[error(transparent)]
    AccountAddress(#[from] AccountAddressParseError),
    #[error(transparent)]
    Bech32(#[from] bech32::Error),
    #[error("{0}")]
    Parse(String),
    #[error(transparent)]
    Subaddress(#[from] SubaddressParseError),
    #[error(transparent)]
    Url(#[from] url::ParseError),
}

/// Defines the available HRPs (human readable prefix) as defined in
/// https://dip.diem.com/dip-5/#format
#[derive(Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub enum HumanReadablePrefix {
    DM,
    PDM,
    TDM,
}

impl std::fmt::Display for HumanReadablePrefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl FromStr for HumanReadablePrefix {
    type Err = IntentIdentifierError;

    fn from_str(s: &str) -> Result<Self, IntentIdentifierError> {
        match s.to_lowercase().as_str() {
            "dm" => Ok(HumanReadablePrefix::DM),
            "tdm" => Ok(HumanReadablePrefix::TDM),
            "pdm" => Ok(HumanReadablePrefix::PDM),
            _ => Err(IntentIdentifierError::Parse(format!("Unknown HRP {}", s))),
        }
    }
}

impl HumanReadablePrefix {
    pub fn as_str(&self) -> &str {
        match self {
            HumanReadablePrefix::DM => "dm",
            HumanReadablePrefix::PDM => "pdm",
            HumanReadablePrefix::TDM => "tdm",
        }
    }
}

/// Encode onchain address and subaddress with human readable prefix (hrp) into bech32 format.
fn encode_account(
    hrp: HumanReadablePrefix,
    account_address: AccountAddress,
    subaddress: Subaddress,
) -> Result<String, bech32::Error> {
    let five_bit_data = [account_address.to_vec(), subaddress.to_vec()]
        .concat()
        .to_base32();
    let diem_encoding_version = vec![u5::try_from_u8(DIEM_BECH32_VERSION)?];
    let bytes = [diem_encoding_version, five_bit_data].concat();
    bech32::encode(hrp.as_str(), bytes, bech32::Variant::Bech32)
}

/// Decodes an encoded address using bech32, ensuring a matching hrp (human readable prefix).
fn decode_account(
    encoded_address: &str,
) -> Result<(HumanReadablePrefix, AccountAddress, Subaddress), IntentIdentifierError> {
    let (hrp_str, data, _variant) = bech32::decode(encoded_address)?;
    if data.len() < DIEM_BECH32_VERSION_LENGTH + AccountAddress::LENGTH + Subaddress::LENGTH {
        return Err(IntentIdentifierError::Parse(format!(
            "Unexpected encoded address length of {}",
            data.len(),
        )));
    }

    let diem_address_version = data[0].to_u8();
    if diem_address_version != DIEM_BECH32_VERSION {
        return Err(IntentIdentifierError::Parse(format!(
            "Unsupported Diem Bech32 Version {}, expected {}",
            diem_address_version, DIEM_BECH32_VERSION,
        )));
    }

    let hrp = HumanReadablePrefix::from_str(hrp_str.as_str())?;
    let payload = Vec::<u8>::from_base32(&data[1..])?;
    let account_address = AccountAddress::from_bytes(&payload[..AccountAddress::LENGTH])?;
    let subaddress = Subaddress::from_bytes(
        &payload[AccountAddress::LENGTH..AccountAddress::LENGTH + Subaddress::LENGTH],
    )?;
    Ok((hrp, account_address, subaddress))
}

/// Encode account identifier string(encoded), currency and amount into
/// Diem intent identifier (https://dip.diem.com/dip-5/)
fn encode_intent(
    encoded_account_identifier: &str,
    currency: Option<Currency>,
    amount: Option<u64>,
) -> String {
    let mut params = Vec::new();
    if let Some(c) = currency {
        params.push(format!("c={}", c));
    }
    if let Some(am) = amount {
        params.push(format!("am={}", am));
    }
    if !params.is_empty() {
        return format!("diem://{}?{}", encoded_account_identifier, params.join("&"));
    }

    format!("diem://{}", encoded_account_identifier)
}

fn normalize_amount(input: Option<&String>) -> Result<Option<u64>, IntentIdentifierError> {
    if let Some(amount) = input {
        if let Ok(parsed_amount) = amount.parse::<u64>() {
            return Ok(Some(parsed_amount));
        } else {
            return Err(IntentIdentifierError::Parse(
                "Amount is invalid".to_string(),
            ));
        }
    }
    Ok(None)
}

fn normalize_currency(input: Option<&String>) -> Result<Option<Currency>, IntentIdentifierError> {
    if let Some(currency) = input {
        if allowed_currency_code_string(currency) {
            return Ok(Some(currency_from_str(currency)?));
        } else {
            return Err(IntentIdentifierError::Parse(format!(
                "currency {} is invalid",
                currency
            )));
        }
    }
    Ok(None)
}

fn currency_from_str(currency: &str) -> Result<Currency, IntentIdentifierError> {
    match currency {
        XUS_NAME => Ok(Currency::XUS),
        XDX_NAME => Ok(Currency::XDX),
        _ => Err(IntentIdentifierError::Parse(format!(
            "Unable to parse currency {}",
            currency,
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bech32::{self, FromBase32, ToBase32};
    use diem_sdk::types::account_address::AccountAddress;
    use rstest::rstest;

    const ACCOUNT_ADDRESS: &str = "f72589b71ff4f8d139674a3f7369c69b";
    const SUBADDRESS: &str = "cf64428bdeb62af2";
    const ZERO_SUBADDRESS: &str = "0000000000000000";

    #[test]
    fn test_encode_intent_parameterless() {
        let intent_id = encode_intent("an_account_identifier", None, None);
        assert_eq!(intent_id, "diem://an_account_identifier");
    }

    #[test]
    fn test_encode_intent_parameters() {
        let intent_id = encode_intent("an_account_identifier", Some(Currency::XUS), Some(45_00));
        assert_eq!(intent_id, "diem://an_account_identifier?c=XUS&am=4500");
    }

    #[test]
    fn test_encode_account() {
        let vasp_address =
            AccountAddress::from_hex_literal("0xd738a0b9851305dfe1d17707f0841dbc").unwrap();
        let user_subaddress = Subaddress::from_hex("9072d012034a880f").unwrap();

        let encoded_account =
            encode_account(HumanReadablePrefix::TDM, vasp_address, user_subaddress).unwrap();

        let expectation = "tdm1p6uu2pwv9zvzalcw3wurlppqahjg895qjqd9gsrcr9dqh8";
        assert_eq!(encoded_account, expectation);
    }

    #[rstest]
    #[case(
        "dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk",
        HumanReadablePrefix::DM,
        ACCOUNT_ADDRESS,
        SUBADDRESS
    )]
    #[case(
        "dm1p7ujcndcl7nudzwt8fglhx6wxnvqqqqqqqqqqqqqd8p9cq",
        HumanReadablePrefix::DM,
        ACCOUNT_ADDRESS,
        ZERO_SUBADDRESS
    )]
    #[case(
        "tdm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4ustv0tyx",
        HumanReadablePrefix::TDM,
        ACCOUNT_ADDRESS,
        SUBADDRESS
    )]
    #[case(
        "tdm1p7ujcndcl7nudzwt8fglhx6wxnvqqqqqqqqqqqqqv88j4s",
        HumanReadablePrefix::TDM,
        ACCOUNT_ADDRESS,
        ZERO_SUBADDRESS
    )]
    fn test_decode_account(
        #[case] encoded_account: &str,
        #[case] expected_hrp: HumanReadablePrefix,
        #[case] expected_account_address: &str,
        #[case] expected_subaddress: &str,
    ) {
        let (hrp, account_address, subaddress) = decode_account(encoded_account).unwrap();
        assert_eq!(hrp, expected_hrp);
        assert_eq!(account_address.to_hex(), expected_account_address);
        assert_eq!(subaddress.to_hex(), expected_subaddress);
    }

    #[rstest]
    #[case(
        "diem://dm1p7ujcndcl7nudzwt8fglhx6wxn08kgs5tm6mz4us2vfufk?c=XUS&am=4500",
        HumanReadablePrefix::DM,
        ACCOUNT_ADDRESS,
        SUBADDRESS,
        Some(Currency::XUS),
        Some(4500)
    )]
    #[case(
        "diem://tdm1p7ujcndcl7nudzwt8fglhx6wxnvqqqqqqqqqqqqqv88j4s",
        HumanReadablePrefix::TDM,
        ACCOUNT_ADDRESS,
        ZERO_SUBADDRESS,
        None,
        None
    )]
    fn test_intent_from_str(
        #[case] encoded_intent_identifier: &str,
        #[case] expected_hrp: HumanReadablePrefix,
        #[case] expected_account_address: &str,
        #[case] expected_subaddress: &str,
        #[case] expected_currency: Option<Currency>,
        #[case] expected_amount: Option<u64>,
    ) {
        let intent = Intent::from_str(encoded_intent_identifier).unwrap();
        assert_eq!(intent.hrp, expected_hrp);
        assert_eq!(intent.account_address.to_hex(), expected_account_address);
        assert_eq!(intent.subaddress.to_hex(), expected_subaddress);
        match intent.currency {
            Some(currency) => assert_eq!(currency, expected_currency.unwrap()),
            None => assert_eq!(None, expected_currency),
        }
        assert_eq!(intent.amount, expected_amount);
    }

    #[test]
    fn test_intent_to_and_from_reversible() {
        let mut hex_address = String::from("0x");
        hex_address.push_str(ACCOUNT_ADDRESS);

        let intent = Intent {
            account_address: AccountAddress::from_hex_literal(&hex_address).unwrap(),
            subaddress: Subaddress::from_hex(SUBADDRESS).unwrap(),
            currency: Some(Currency::XUS),
            amount: Some(4500),
            hrp: HumanReadablePrefix::DM,
        };

        let encoded_intent = intent.to_encoded_string().unwrap();
        let decoded_intent = Intent::from_str(&encoded_intent).unwrap();

        assert_eq!(
            decoded_intent.account_address.to_hex(),
            intent.account_address.to_hex()
        );
        assert_eq!(
            decoded_intent.subaddress.to_hex(),
            intent.subaddress.to_hex()
        );
        assert_eq!(decoded_intent.currency, intent.currency);
        assert_eq!(decoded_intent.amount, intent.amount);
        assert_eq!(decoded_intent.hrp, intent.hrp);
    }

    #[test]
    fn test_bech32_reversibility() {
        let hrp = "dm";
        let address_str = "deadbeef";
        let data = hex::decode(address_str).unwrap().to_base32();
        let encoded = bech32::encode(hrp, data, bech32::Variant::Bech32).unwrap();
        assert_eq!(encoded, "dm1m6kmamc4w6d7n");

        let (decoded_hrp, decoded_data, _variant) = bech32::decode(encoded.as_str()).unwrap();
        assert_eq!(hrp, decoded_hrp);

        let decoded_address = Vec::<u8>::from_base32(&decoded_data).unwrap();
        assert_eq!(hex::encode(decoded_address), address_str);
    }
}

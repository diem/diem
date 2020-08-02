// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    account_config::{
        type_tag_for_currency_code, AccountResource, AccountRole, BalanceResource, ChainIdResource,
        ChildVASP, Credential, CurrencyInfoResource, DesignatedDealer, FreezingBit, ParentVASP,
        PreburnResource, ACCOUNT_RECEIVED_EVENT_PATH, ACCOUNT_SENT_EVENT_PATH,
    },
    block_metadata::{LibraBlockResource, NEW_BLOCK_EVENT_PATH},
    event::EventHandle,
    libra_timestamp::LibraTimestampResource,
    on_chain_config::{
        ConfigurationResource, LibraVersion, OnChainConfig, RegisteredCurrencies, ValidatorSet,
    },
    validator_config::{ValidatorConfigResource, ValidatorOperatorConfigResource},
};
use anyhow::{bail, Error, Result};
use move_core_types::{identifier::Identifier, move_resource::MoveResource};
use serde::{de::DeserializeOwned, export::Formatter, Deserialize, Serialize};
use std::{collections::btree_map::BTreeMap, convert::TryFrom, fmt};

#[derive(Default, Deserialize, PartialEq, Serialize)]
pub struct AccountState(BTreeMap<Vec<u8>, Vec<u8>>);

impl AccountState {
    // By design and do not remove
    pub fn get_account_address(&self) -> Result<Option<AccountAddress>> {
        self.get_account_resource()
            .map(|opt_ar| opt_ar.map(|ar| ar.sent_events().key().get_creator_address()))
    }

    pub fn get_account_resource(&self) -> Result<Option<AccountResource>> {
        self.get_resource(&AccountResource::resource_path())
    }

    pub fn get_balance_resources(
        &self,
        currency_codes: &[Identifier],
    ) -> Result<BTreeMap<Identifier, BalanceResource>> {
        currency_codes
            .iter()
            .filter_map(|currency_code| {
                let currency_type_tag = type_tag_for_currency_code(currency_code.to_owned());
                // TODO: update this to use BalanceResource::resource_path once that takes type
                // parameters
                self.get_resource(&BalanceResource::access_path_for(currency_type_tag))
                    .transpose()
                    .map(|balance| balance.map(|b| (currency_code.to_owned(), b)))
            })
            .collect()
    }

    pub fn get_preburn_balances(
        &self,
        currency_codes: &[Identifier],
    ) -> Result<BTreeMap<Identifier, PreburnResource>> {
        currency_codes
            .iter()
            .filter_map(|currency_code| {
                let currency_type_tag = type_tag_for_currency_code(currency_code.to_owned());
                // TODO: update this to use PreburnResource::resource_path once that takes type
                // parameters
                self.get_resource(&PreburnResource::access_path_for(currency_type_tag))
                    .transpose()
                    .map(|preburn_balance| preburn_balance.map(|b| (currency_code.to_owned(), b)))
            })
            .collect()
    }

    pub fn get_chain_id_resource(&self) -> Result<Option<ChainIdResource>> {
        self.get_resource(&ChainIdResource::resource_path())
    }

    pub fn get_configuration_resource(&self) -> Result<Option<ConfigurationResource>> {
        self.get_resource(&ConfigurationResource::resource_path())
    }

    pub fn get_libra_timestamp_resource(&self) -> Result<Option<LibraTimestampResource>> {
        self.get_resource(&LibraTimestampResource::resource_path())
    }

    pub fn get_validator_config_resource(&self) -> Result<Option<ValidatorConfigResource>> {
        self.get_resource(&ValidatorConfigResource::resource_path())
    }

    pub fn get_validator_operator_config_resource(
        &self,
    ) -> Result<Option<ValidatorOperatorConfigResource>> {
        self.get_resource(&ValidatorOperatorConfigResource::resource_path())
    }

    pub fn get_freezing_bit(&self) -> Result<Option<FreezingBit>> {
        self.get_resource(&FreezingBit::resource_path())
    }

    pub fn get_account_role(&self, currency_codes: &[Identifier]) -> Result<Option<AccountRole>> {
        if self.0.contains_key(&ParentVASP::resource_path()) {
            match (
                self.get_resource(&ParentVASP::resource_path()),
                self.get_resource(&Credential::resource_path()),
            ) {
                (Ok(Some(vasp)), Ok(Some(credential))) => {
                    Ok(Some(AccountRole::ParentVASP { vasp, credential }))
                }
                _ => Ok(None),
            }
        } else if self.0.contains_key(&ChildVASP::resource_path()) {
            self.get_resource(&ChildVASP::resource_path())
                .map(|r_opt| r_opt.map(AccountRole::ChildVASP))
        } else if self.0.contains_key(&DesignatedDealer::resource_path()) {
            match (
                self.get_resource(&Credential::resource_path()),
                self.get_preburn_balances(&currency_codes),
                self.get_resource(&DesignatedDealer::resource_path()),
            ) {
                (Ok(Some(dd_credential)), Ok(preburn_balances), Ok(Some(designated_dealer))) => {
                    Ok(Some(AccountRole::DesignatedDealer {
                        dd_credential,
                        preburn_balances,
                        designated_dealer,
                    }))
                }
                _ => Ok(None),
            }
        } else {
            // TODO: add role_id to Unknown
            Ok(Some(AccountRole::Unknown))
        }
    }

    pub fn get_validator_set(&self) -> Result<Option<ValidatorSet>> {
        self.get_resource(&ValidatorSet::CONFIG_ID.access_path().path)
    }

    pub fn get_libra_version(&self) -> Result<Option<LibraVersion>> {
        self.get_resource(&LibraVersion::CONFIG_ID.access_path().path)
    }

    pub fn get_registered_currency_info_resources(
        &self,
    ) -> Result<Vec<Option<CurrencyInfoResource>>> {
        let path = RegisteredCurrencies::CONFIG_ID.access_path().path;
        let currencies: Option<RegisteredCurrencies> = self.get_resource(&path)?;
        match currencies {
            Some(currencies) => currencies
                .currency_codes()
                .iter()
                .map(|code| CurrencyInfoResource::resource_path_for(code.clone()))
                .map(|access_path| self.get_resource(&access_path.path))
                .collect(),
            None => Ok(vec![]),
        }
    }

    pub fn get_libra_block_resource(&self) -> Result<Option<LibraBlockResource>> {
        self.get_resource(&LibraBlockResource::resource_path())
    }

    pub fn get_event_handle_by_query_path(&self, query_path: &[u8]) -> Result<Option<EventHandle>> {
        let event_handle = if *ACCOUNT_RECEIVED_EVENT_PATH == query_path {
            self.get_account_resource()?
                .map(|account_resource| account_resource.received_events().clone())
        } else if *ACCOUNT_SENT_EVENT_PATH == query_path {
            self.get_account_resource()?
                .map(|account_resource| account_resource.sent_events().clone())
        } else if *NEW_BLOCK_EVENT_PATH == query_path {
            self.get_libra_block_resource()?
                .map(|libra_block_resource| libra_block_resource.new_block_events().clone())
        } else {
            bail!("Unrecognized query path: {:?}", query_path);
        };

        Ok(event_handle)
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.0.get(key)
    }

    pub fn get_resource<T: DeserializeOwned>(&self, key: &[u8]) -> Result<Option<T>> {
        self.0
            .get(key)
            .map(|bytes| lcs::from_bytes(bytes))
            .transpose()
            .map_err(Into::into)
    }

    pub fn insert(&mut self, key: Vec<u8>, value: Vec<u8>) -> Option<Vec<u8>> {
        self.0.insert(key, value)
    }

    pub fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>> {
        self.0.remove(key)
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item = (&Vec<u8>, &Vec<u8>)> {
        self.0.iter()
    }
}

impl fmt::Debug for AccountState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: add support for other types of resources
        let account_resource_str = self
            .get_account_resource()
            .map(|account_resource_opt| format!("{:#?}", account_resource_opt))
            .unwrap_or_else(|e| format!("parse error: {:#?}", e));

        let libra_timestamp_str = self
            .get_libra_timestamp_resource()
            .map(|libra_timestamp_opt| format!("{:#?}", libra_timestamp_opt))
            .unwrap_or_else(|e| format!("parse: {:#?}", e));

        let validator_config_str = self
            .get_validator_config_resource()
            .map(|validator_config_opt| format!("{:#?}", validator_config_opt))
            .unwrap_or_else(|e| format!("parse error: {:#?}", e));

        let validator_set_str = self
            .get_validator_set()
            .map(|validator_set_opt| format!("{:#?}", validator_set_opt))
            .unwrap_or_else(|e| format!("parse error: {:#?}", e));

        write!(
            f,
            "{{ \n \
             AccountResource {{ {} }} \n \
             LibraTimestamp {{ {} }} \n \
             ValidatorConfig {{ {} }} \n \
             ValidatorSet {{ {} }} \n \
             }}",
            account_resource_str, libra_timestamp_str, validator_config_str, validator_set_str,
        )
    }
}

impl TryFrom<(&AccountResource, &BalanceResource)> for AccountState {
    type Error = Error;

    fn try_from(
        (account_resource, balance_resource): (&AccountResource, &BalanceResource),
    ) -> Result<Self> {
        let mut btree_map: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        btree_map.insert(
            AccountResource::resource_path(),
            lcs::to_bytes(account_resource)?,
        );
        btree_map.insert(
            BalanceResource::resource_path(),
            lcs::to_bytes(balance_resource)?,
        );

        Ok(Self(btree_map))
    }
}

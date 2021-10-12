// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::Path,
    account_address::AccountAddress,
    account_config::{
        currency_code_from_type_tag, AccountResource, AccountRole, BalanceResource, CRSNResource,
        ChainIdResource, ChildVASP, Credential, CurrencyInfoResource, DesignatedDealer,
        DesignatedDealerPreburns, FreezingBit, ParentVASP, PreburnQueueResource, PreburnResource,
        VASPDomainManager, VASPDomains,
    },
    block_metadata::DiemBlockResource,
    diem_timestamp::DiemTimestampResource,
    on_chain_config::{
        ConfigurationResource, DiemVersion, OnChainConfig, RegisteredCurrencies,
        VMPublishingOption, ValidatorSet,
    },
    validator_config::{ValidatorConfigResource, ValidatorOperatorConfigResource},
};
use anyhow::{format_err, Error, Result};
use move_core_types::{
    identifier::Identifier,
    language_storage::{StructTag, CORE_CODE_ADDRESS},
    move_resource::MoveResource,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
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
        self.get_resource::<AccountResource>()
    }

    pub fn get_crsn_resource(&self) -> Result<Option<CRSNResource>> {
        self.get_resource::<CRSNResource>()
    }

    pub fn get_balance_resources(&self) -> Result<BTreeMap<Identifier, BalanceResource>> {
        self.get_resources_with_type::<BalanceResource>()
            .map(|maybe_resource| {
                let (struct_tag, resource) = maybe_resource?;
                let currency_code = collect_exactly_one(struct_tag.type_params.into_iter())
                    .ok_or_else(|| format_err!("expected one currency_code type tag"))
                    .and_then(currency_code_from_type_tag)?;
                Ok((currency_code, resource))
            })
            .collect()
    }

    pub fn get_preburn_balances(&self) -> Result<BTreeMap<Identifier, PreburnResource>> {
        self.get_resources_with_type::<PreburnResource>()
            .map(|maybe_resource| {
                let (struct_tag, resource) = maybe_resource?;
                let currency_code = collect_exactly_one(struct_tag.type_params.into_iter())
                    .ok_or_else(|| format_err!("expected one currency_code type tag"))
                    .and_then(currency_code_from_type_tag)?;
                Ok((currency_code, resource))
            })
            .collect()
    }

    pub fn get_preburn_queue_balances(&self) -> Result<BTreeMap<Identifier, PreburnQueueResource>> {
        self.get_resources_with_type::<PreburnQueueResource>()
            .map(|maybe_resource| {
                let (struct_tag, resource) = maybe_resource?;
                let currency_code = collect_exactly_one(struct_tag.type_params.into_iter())
                    .ok_or_else(|| format_err!("expected one currency_code type tag"))
                    .and_then(currency_code_from_type_tag)?;
                Ok((currency_code, resource))
            })
            .collect()
    }

    pub fn get_chain_id_resource(&self) -> Result<Option<ChainIdResource>> {
        self.get_resource::<ChainIdResource>()
    }

    pub fn get_configuration_resource(&self) -> Result<Option<ConfigurationResource>> {
        self.get_resource::<ConfigurationResource>()
    }

    pub fn get_diem_timestamp_resource(&self) -> Result<Option<DiemTimestampResource>> {
        self.get_resource::<DiemTimestampResource>()
    }

    pub fn get_validator_config_resource(&self) -> Result<Option<ValidatorConfigResource>> {
        self.get_resource::<ValidatorConfigResource>()
    }

    pub fn get_validator_operator_config_resource(
        &self,
    ) -> Result<Option<ValidatorOperatorConfigResource>> {
        self.get_resource::<ValidatorOperatorConfigResource>()
    }

    pub fn get_freezing_bit(&self) -> Result<Option<FreezingBit>> {
        self.get_resource::<FreezingBit>()
    }

    pub fn get_account_role(&self) -> Result<Option<AccountRole>> {
        if self.0.contains_key(&ParentVASP::resource_path()) {
            match (
                self.get_resource::<ParentVASP>(),
                self.get_resource::<Credential>(),
                self.get_resource::<VASPDomains>(),
            ) {
                (Ok(Some(vasp)), Ok(Some(credential)), Ok(vasp_domains)) => {
                    Ok(Some(AccountRole::ParentVASP {
                        vasp,
                        credential,
                        vasp_domains,
                    }))
                }
                _ => Ok(None),
            }
        } else if self.0.contains_key(&ChildVASP::resource_path()) {
            self.get_resource::<ChildVASP>()
                .map(|r_opt| r_opt.map(AccountRole::ChildVASP))
        } else if self.0.contains_key(&DesignatedDealer::resource_path()) {
            match (
                self.get_resource::<Credential>(),
                self.get_preburn_balances(),
                self.get_preburn_queue_balances(),
                self.get_resource::<DesignatedDealer>(),
            ) {
                (
                    Ok(Some(dd_credential)),
                    Ok(preburn_balances),
                    Ok(preburn_queues),
                    Ok(Some(designated_dealer)),
                ) => {
                    let preburn_balances =
                        if preburn_balances.is_empty() && !preburn_queues.is_empty() {
                            DesignatedDealerPreburns::PreburnQueue(preburn_queues)
                        } else if !preburn_balances.is_empty() && preburn_queues.is_empty() {
                            DesignatedDealerPreburns::Preburn(preburn_balances)
                        } else {
                            return Ok(None);
                        };
                    Ok(Some(AccountRole::DesignatedDealer {
                        dd_credential,
                        preburn_balances,
                        designated_dealer,
                    }))
                }
                _ => Ok(None),
            }
        } else if self.0.contains_key(&VASPDomainManager::resource_path()) {
            match self.get_resource::<VASPDomainManager>() {
                Ok(Some(vasp_domain_manager)) => Ok(Some(AccountRole::TreasuryCompliance {
                    vasp_domain_manager,
                })),
                _ => Ok(None),
            }
        } else {
            // TODO: add role_id to Unknown
            Ok(Some(AccountRole::Unknown))
        }
    }

    pub fn get_validator_set(&self) -> Result<Option<ValidatorSet>> {
        self.get_config::<ValidatorSet>()
    }

    pub fn get_diem_version(&self) -> Result<Option<DiemVersion>> {
        self.get_config::<DiemVersion>()
    }

    pub fn get_vm_publishing_option(&self) -> Result<Option<VMPublishingOption>> {
        self.0
            .get(&VMPublishingOption::CONFIG_ID.access_path().path)
            .map(|bytes| VMPublishingOption::deserialize_into_config(bytes))
            .transpose()
            .map_err(Into::into)
    }

    pub fn get_registered_currency_info_resources(&self) -> Result<Vec<CurrencyInfoResource>> {
        let currencies: Option<RegisteredCurrencies> = self.get_config()?;
        match currencies {
            Some(currencies) => {
                let codes = currencies.currency_codes();
                let mut resources = vec![];
                for code in codes {
                    let access_path = CurrencyInfoResource::resource_path_for(code.clone());
                    let info: CurrencyInfoResource = self
                        .get_resource_impl(&access_path.path)?
                        .ok_or_else(|| format_err!("currency info resource not found: {}", code))?;
                    resources.push(info);
                }
                Ok(resources)
            }
            None => Ok(vec![]),
        }
    }

    pub fn get_diem_block_resource(&self) -> Result<Option<DiemBlockResource>> {
        self.get_resource::<DiemBlockResource>()
    }

    pub fn get(&self, key: &[u8]) -> Option<&Vec<u8>> {
        self.0.get(key)
    }

    pub fn get_resource_impl<T: DeserializeOwned>(&self, key: &[u8]) -> Result<Option<T>> {
        self.0
            .get(key)
            .map(|bytes| bcs::from_bytes(bytes))
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

    pub fn get_config<T: OnChainConfig>(&self) -> Result<Option<T>> {
        self.get_resource_impl(&T::CONFIG_ID.access_path().path)
    }

    pub fn get_resource<T: MoveResource>(&self) -> Result<Option<T>> {
        self.get_resource_impl(&T::struct_tag().access_vector())
    }

    /// Return an iterator over the module values stored under this account
    pub fn get_modules(&self) -> impl Iterator<Item = &Vec<u8>> {
        self.0.iter().filter_map(
            |(k, v)| match Path::try_from(k).expect("Invalid access path") {
                Path::Code(_) => Some(v),
                Path::Resource(_) => None,
            },
        )
    }

    /// Into an iterator over the module values stored under this account
    pub fn into_modules(self) -> impl Iterator<Item = Vec<u8>> {
        self.0.into_iter().filter_map(|(k, v)| {
            match Path::try_from(&k).expect("Invalid access path") {
                Path::Code(_) => Some(v),
                Path::Resource(_) => None,
            }
        })
    }

    /// Return an iterator over all resources stored under this account.
    ///
    /// Note that resource access [`Path`]s that fail to deserialize will be
    /// silently ignored.
    pub fn get_resources(&self) -> impl Iterator<Item = (StructTag, &[u8])> {
        self.0.iter().filter_map(|(k, v)| match Path::try_from(k) {
            Ok(Path::Resource(struct_tag)) => Some((struct_tag, v.as_ref())),
            Ok(Path::Code(_)) | Err(_) => None,
        })
    }

    /// Given a particular `MoveResource`, return an iterator with all instances
    /// of that resource (there may be multiple with different generic type parameters).
    pub fn get_resources_with_type<T: MoveResource>(
        &self,
    ) -> impl Iterator<Item = Result<(StructTag, T)>> + '_ {
        self.get_resources().filter_map(|(struct_tag, bytes)| {
            let matches_resource = struct_tag.address == CORE_CODE_ADDRESS
                && struct_tag.module.as_ref() == T::MODULE_NAME
                && struct_tag.name.as_ref() == T::STRUCT_NAME;
            if matches_resource {
                match bcs::from_bytes::<T>(bytes) {
                    Ok(resource) => Some(Ok((struct_tag, resource))),
                    Err(err) => Some(Err(format_err!(
                        "failed to deserialize resource: '{}', error: {:?}",
                        struct_tag,
                        err
                    ))),
                }
            } else {
                None
            }
        })
    }
}

impl fmt::Debug for AccountState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // TODO: add support for other types of resources
        let account_resource_str = self
            .get_account_resource()
            .map(|account_resource_opt| format!("{:#?}", account_resource_opt))
            .unwrap_or_else(|e| format!("parse error: {:#?}", e));

        let diem_timestamp_str = self
            .get_diem_timestamp_resource()
            .map(|diem_timestamp_opt| format!("{:#?}", diem_timestamp_opt))
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
             DiemTimestamp {{ {} }} \n \
             ValidatorConfig {{ {} }} \n \
             ValidatorSet {{ {} }} \n \
             }}",
            account_resource_str, diem_timestamp_str, validator_config_str, validator_set_str,
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
            bcs::to_bytes(account_resource)?,
        );
        btree_map.insert(
            BalanceResource::resource_path(),
            bcs::to_bytes(balance_resource)?,
        );

        Ok(Self(btree_map))
    }
}

/// If an iterator contains exactly one item, then return it. Otherwise return
/// `None` if there are no items or more than one items.
fn collect_exactly_one<T>(iter: impl Iterator<Item = T>) -> Option<T> {
    let mut iter = iter.fuse();
    match (iter.next(), iter.next()) {
        (Some(item), None) => Some(item),
        _ => None,
    }
}

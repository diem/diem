// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress, account_config::AccountResource, account_state::AccountState,
    event::EventKey, ledger_info::LedgerInfo, proof::AccountStateProof, transaction::Version,
};
use anyhow::{anyhow, ensure, format_err, Error, Result};
use libra_crypto::{
    hash::{CryptoHash, CryptoHasher},
    HashValue,
};
use libra_crypto_derive::CryptoHasher;
#[cfg(any(test, feature = "fuzzing"))]
use proptest::{arbitrary::Arbitrary, prelude::*};
#[cfg(any(test, feature = "fuzzing"))]
use proptest_derive::Arbitrary;
use serde::{Deserialize, Serialize};
use std::{
    convert::{TryFrom, TryInto},
    fmt,
};

#[derive(Clone, Eq, PartialEq, Serialize, Deserialize, CryptoHasher)]
pub struct AccountStateBlob {
    blob: Vec<u8>,
}

impl fmt::Debug for AccountStateBlob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decoded = lcs::from_bytes(&self.blob)
            .map(|account_state: AccountState| format!("{:#?}", account_state))
            .unwrap_or_else(|_| String::from("[fail]"));

        write!(
            f,
            "AccountStateBlob {{ \n \
             Raw: 0x{} \n \
             Decoded: {} \n \
             }}",
            hex::encode(&self.blob),
            decoded,
        )
    }
}

impl AsRef<[u8]> for AccountStateBlob {
    fn as_ref(&self) -> &[u8] {
        &self.blob
    }
}

impl From<AccountStateBlob> for Vec<u8> {
    fn from(account_state_blob: AccountStateBlob) -> Vec<u8> {
        account_state_blob.blob
    }
}

impl From<Vec<u8>> for AccountStateBlob {
    fn from(blob: Vec<u8>) -> AccountStateBlob {
        AccountStateBlob { blob }
    }
}

impl TryFrom<crate::proto::types::AccountStateBlob> for AccountStateBlob {
    type Error = Error;

    fn try_from(proto: crate::proto::types::AccountStateBlob) -> Result<Self> {
        Ok(proto.blob.into())
    }
}

impl From<AccountStateBlob> for crate::proto::types::AccountStateBlob {
    fn from(blob: AccountStateBlob) -> Self {
        Self { blob: blob.blob }
    }
}

impl TryFrom<&AccountState> for AccountStateBlob {
    type Error = Error;

    fn try_from(account_state: &AccountState) -> Result<Self> {
        Ok(Self {
            blob: lcs::to_bytes(account_state)?,
        })
    }
}

impl TryFrom<&AccountStateBlob> for AccountState {
    type Error = Error;

    fn try_from(account_state_blob: &AccountStateBlob) -> Result<Self> {
        lcs::from_bytes(&account_state_blob.blob).map_err(Into::into)
    }
}

impl TryFrom<&AccountResource> for AccountStateBlob {
    type Error = Error;

    fn try_from(account_resource: &AccountResource) -> Result<Self> {
        Self::try_from(&AccountState::try_from(account_resource)?)
    }
}

impl TryFrom<&AccountStateBlob> for AccountResource {
    type Error = Error;

    fn try_from(account_state_blob: &AccountStateBlob) -> Result<Self> {
        AccountState::try_from(account_state_blob)?
            .get_account_resource()?
            .ok_or_else(|| anyhow!("AccountResource not found."))
    }
}

impl CryptoHash for AccountStateBlob {
    type Hasher = AccountStateBlobHasher;

    fn hash(&self) -> HashValue {
        let mut hasher = Self::Hasher::default();
        hasher.write(&self.blob);
        hasher.finish()
    }
}

#[cfg(any(test, feature = "fuzzing"))]
prop_compose! {
    fn account_state_blob_strategy()(account_resource in any::<AccountResource>()) -> AccountStateBlob {
        AccountStateBlob::try_from(&account_resource).unwrap()
    }
}

#[cfg(any(test, feature = "fuzzing"))]
impl Arbitrary for AccountStateBlob {
    type Parameters = ();
    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        account_state_blob_strategy().boxed()
    }

    type Strategy = BoxedStrategy<Self>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "fuzzing"), derive(Arbitrary))]
pub struct AccountStateWithProof {
    /// The transaction version at which this account state is seen.
    pub version: Version,
    /// Blob value representing the account state. If this field is not set, it
    /// means the account does not exist.
    pub blob: Option<AccountStateBlob>,
    /// The proof the client can use to authenticate the value.
    pub proof: AccountStateProof,
}

impl AccountStateWithProof {
    /// Constructor.
    pub fn new(version: Version, blob: Option<AccountStateBlob>, proof: AccountStateProof) -> Self {
        Self {
            version,
            blob,
            proof,
        }
    }

    /// Verifies the the account state blob with the proof, both carried by `self`.
    ///
    /// Two things are ensured if no error is raised:
    ///   1. This account state exists in the ledger represented by `ledger_info`.
    ///   2. It belongs to account of `address` and is seen at the time the transaction at version
    /// `state_version` is just committed. To make sure this is the latest state, pass in
    /// `ledger_info.version()` as `state_version`.
    pub fn verify(
        &self,
        ledger_info: &LedgerInfo,
        version: Version,
        address: AccountAddress,
    ) -> Result<()> {
        ensure!(
            self.version == version,
            "State version ({}) is not expected ({}).",
            self.version,
            version,
        );

        self.proof
            .verify(ledger_info, version, address.hash(), self.blob.as_ref())
    }

    /// Returns the `EventKey` (if existent) and number of total events for
    /// an event stream specified by a query path.
    ///
    /// If the resource referred by the path that is supposed to hold the `EventHandle`
    /// doesn't exist, returns (None, 0). While if the path is invalid, raises error.
    ///
    /// For example:
    ///   1. if asked for DiscoverySetChange event from an ordinary user account,
    /// this returns (None, 0)
    ///   2. but if asked for a random path that we don't understand, it's an error.
    pub fn get_event_key_and_count_by_query_path(
        &self,
        path: &[u8],
    ) -> Result<(Option<EventKey>, u64)> {
        if let Some(account_blob) = &self.blob {
            if let Some(event_handle) =
                AccountState::try_from(account_blob)?.get_event_handle_by_query_path(path)?
            {
                Ok((Some(*event_handle.key()), event_handle.count()))
            } else {
                Ok((None, 0))
            }
        } else {
            Ok((None, 0))
        }
    }
}

impl TryFrom<crate::proto::types::AccountStateWithProof> for AccountStateWithProof {
    type Error = Error;

    fn try_from(mut proto: crate::proto::types::AccountStateWithProof) -> Result<Self> {
        Ok(Self::new(
            proto.version,
            proto
                .blob
                .take()
                .map(AccountStateBlob::try_from)
                .transpose()?,
            proto
                .proof
                .ok_or_else(|| format_err!("Missing proof"))?
                .try_into()?,
        ))
    }
}

impl From<AccountStateWithProof> for crate::proto::types::AccountStateWithProof {
    fn from(account: AccountStateWithProof) -> Self {
        Self {
            version: account.version,
            blob: account.blob.map(Into::into),
            proof: Some(account.proof.into()),
        }
    }
}

#[cfg(test)]
mod account_state_blob_test;

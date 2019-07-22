// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unit_arg)]

#[cfg(any(test, feature = "testing"))]
use crate::account_config::{account_resource_path, AccountResource};
use crate::{
    account_address::AccountAddress,
    account_config::get_account_resource_or_default,
    ledger_info::LedgerInfo,
    proof::{verify_account_state, AccountStateProof},
    transaction::Version,
};

use canonical_serialization::{SimpleDeserializer, SimpleSerializer};
use crypto::{
    hash::{AccountStateBlobHasher, CryptoHash, CryptoHasher},
    HashValue,
};
use failure::prelude::*;
#[cfg(any(test, feature = "testing"))]
use proptest::{arbitrary::Arbitrary, prelude::*};
#[cfg(any(test, feature = "testing"))]
use proptest_derive::Arbitrary;
use proto_conv::{FromProto, IntoProto};
use std::{collections::BTreeMap, convert::TryFrom, fmt};

#[derive(Clone, Eq, PartialEq, FromProto, IntoProto)]
#[ProtoType(crate::proto::account_state_blob::AccountStateBlob)]
pub struct AccountStateBlob {
    blob: Vec<u8>,
}

impl fmt::Debug for AccountStateBlob {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let decoded = get_account_resource_or_default(&Some(self.clone()))
            .map(|resource| format!("{:#?}", resource))
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

impl TryFrom<&BTreeMap<Vec<u8>, Vec<u8>>> for AccountStateBlob {
    type Error = failure::Error;

    fn try_from(map: &BTreeMap<Vec<u8>, Vec<u8>>) -> Result<Self> {
        Ok(Self {
            blob: SimpleSerializer::serialize(map)?,
        })
    }
}

impl TryFrom<&AccountStateBlob> for BTreeMap<Vec<u8>, Vec<u8>> {
    type Error = failure::Error;

    fn try_from(account_state_blob: &AccountStateBlob) -> Result<Self> {
        SimpleDeserializer::deserialize(&account_state_blob.blob)
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

#[cfg(any(test, feature = "testing"))]
prop_compose! {
    pub fn account_state_blob_strategy()(account_resource in any::<AccountResource>()) -> AccountStateBlob {
        let mut account_state: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
        account_state.insert(
            account_resource_path(),
            SimpleSerializer::<Vec<u8>>::serialize(&account_resource).unwrap(),
        );
        AccountStateBlob::try_from(&account_state).unwrap()
    }
}

#[cfg(any(test, feature = "testing"))]
impl Arbitrary for AccountStateBlob {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        account_state_blob_strategy().boxed()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(any(test, feature = "testing"), derive(Arbitrary))]
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

        verify_account_state(
            ledger_info,
            version,
            address.hash(),
            &self.blob,
            &self.proof,
        )
    }
}

impl FromProto for AccountStateWithProof {
    type ProtoType = crate::proto::account_state_blob::AccountStateWithProof;

    fn from_proto(mut object: Self::ProtoType) -> Result<Self> {
        Ok(AccountStateWithProof {
            version: object.get_version(),
            blob: object
                .blob
                .take()
                .map(AccountStateBlob::from_proto)
                .transpose()?,
            proof: AccountStateProof::from_proto(object.take_proof())?,
        })
    }
}

impl IntoProto for AccountStateWithProof {
    type ProtoType = crate::proto::account_state_blob::AccountStateWithProof;

    fn into_proto(self) -> Self::ProtoType {
        let mut out = Self::ProtoType::new();
        out.set_version(self.version);
        if let Some(blob) = self.blob {
            out.set_blob(blob.into_proto());
        }
        out.set_proof(self.proof.into_proto());
        out
    }
}

#[cfg(test)]
mod account_state_blob_test;

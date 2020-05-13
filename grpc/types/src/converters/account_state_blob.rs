// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{format_err, Error, Result};
use std::convert::{TryFrom, TryInto};

use libra_types::account_state_blob::{AccountStateBlob, AccountStateWithProof};

impl TryFrom<crate::proto::types::AccountStateBlob> for AccountStateBlob {
    type Error = Error;

    fn try_from(proto: crate::proto::types::AccountStateBlob) -> Result<Self> {
        Ok(proto.blob.into())
    }
}

impl From<AccountStateBlob> for crate::proto::types::AccountStateBlob {
    fn from(blob: AccountStateBlob) -> Self {
        Self { blob: blob.into() }
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

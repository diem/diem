// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{ensure, format_err, Context as _, Result};
use libra_config::config::RoleType;
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_network_address::NetworkAddress;
use libra_types::{
    account_config,
    account_state::AccountState,
    account_state_blob::AccountStateWithProof,
    epoch_change::EpochChangeProof,
    ledger_info::LedgerInfoWithSignatures,
    on_chain_config::ValidatorSet,
    proof::AccumulatorConsistencyProof,
    trusted_state::{TrustedState, TrustedStateChange},
    validator_info::ValidatorInfo,
    PeerId,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

/// OnchainDiscovery LibraNet message types. These are sent over-the-wire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OnchainDiscoveryMsg {
    QueryDiscoverySetRequest(QueryDiscoverySetRequest),
    QueryDiscoverySetResponse(Box<QueryDiscoverySetResponse>),
}

/// A request for another peer's latest validator set and a validator change proof
/// to get the client up-to-date if they're behind.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryDiscoverySetRequest {
    pub known_version: u64,
    // TODO(philiphayes): split sets into separate V and VFN sets. add enum to
    // select by discovery set type.
}

/// A response to a [`QueryDiscoverySetRequest`]. The server will include an
/// epoch change proof if the client is behind.
///
/// The validator set only changes when there is a new epoch. To minimize
/// wire overhead, the server will include the validator set account's
/// [`AccountStateWithProof`] _if and only if_ the server also presents a
/// non-empty epoch change proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryDiscoverySetResponse {
    pub latest_li: LedgerInfoWithSignatures,
    pub epoch_change_proof: EpochChangeProof,
    pub accumulator_proof: AccumulatorConsistencyProof,
    pub account_state: Option<AccountStateWithProof>,
}

impl QueryDiscoverySetResponse {
    /// Verify and ratchet the given trusted state, returning the verified trusted
    /// state change and latest validator set (if present).
    ///
    /// 1. Verify and ratchet trusted state using epoch changes and latest ledger info
    /// 2. (If present) Verify validator set account state proof
    /// 3. (If present) Deserialize `ValidatorSet` from validator set resource in account blob
    pub fn verify_and_ratchet<'a>(
        &'a self,
        req_msg: &QueryDiscoverySetRequest,
        trusted_state: &'a TrustedState,
    ) -> Result<(TrustedStateChange<'a>, Option<ValidatorSet>)> {
        // TODO(philiphayes): how to deal with partial epoch change proofs?
        // should probably not return discovery set until at head?

        let has_epoch_change = !self.epoch_change_proof.ledger_info_with_sigs.is_empty();
        let has_validator_set = self.account_state.is_some();

        // enforce property: epoch change <==> some validator set in response
        ensure!(
            has_epoch_change == has_validator_set,
            "mismatch between epoch change and validator set. \
             has_epoch_change iff has_validator_set: \
             has_epoch_change: {}, has_validator_set: {}",
            has_epoch_change,
            has_validator_set,
        );

        // check response is not stale
        let ledger_version = self.latest_li.ledger_info().version();
        ensure!(
            ledger_version >= req_msg.known_version,
            "received stale ledger_info: ledger_version: {}, request known_version: {}",
            ledger_version,
            req_msg.known_version,
        );

        // try to ratchet trusted state
        let trusted_state_change = trusted_state
            .verify_and_ratchet(&self.latest_li, &self.epoch_change_proof)
            .context("failed to ratchet trusted_state")?;

        // if the response contains the validator set account, then verify
        // account_state_proof and pull out the validator set
        let opt_validator_set = self
            .account_state
            .as_ref()
            .map(|account_state| -> Result<ValidatorSet> {
                account_state
                    .verify(
                        self.latest_li.ledger_info(),
                        ledger_version,
                        account_config::validator_set_address(),
                    )
                    .context("failed to verify account state proof for validator set resource")?;

                let blob = account_state
                    .blob
                    .as_ref()
                    .ok_or_else(|| format_err!("validator set account blob cannot be missing"))?;

                let validator_set = AccountState::try_from(blob)?
                    .get_validator_set()?
                    .ok_or_else(|| format_err!("validator set resource cannot be missing"))?;
                Ok(validator_set)
            })
            .transpose()?;

        // TODO(philiphayes): do something with accumulator_proof?
        // self.query_res.accumulator_proof.

        Ok((trusted_state_change, opt_validator_set))
    }
}

/// An internal representation of the DiscoverySet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoverySetInternal(pub HashMap<PeerId, DiscoveryInfoInternal>);

impl DiscoverySetInternal {
    pub fn from_validator_set(role_filter: RoleType, validator_set: ValidatorSet) -> Self {
        Self(
            validator_set
                .into_iter()
                .filter_map(|validator_info| {
                    let peer_id = *validator_info.account_address();
                    let res_info =
                        DiscoveryInfoInternal::try_from_validator_info(role_filter, validator_info);

                    // ignore network addresses that fail to deserialize
                    res_info
                        .map_err(|err| {
                            debug!(
                                "failed to deserialize addresses from validator: {}, err: {}",
                                peer_id.short_str(),
                                err
                            )
                        })
                        .map(|info| (peer_id, info))
                        .ok()
                })
                .collect::<HashMap<_, _>>(),
        )
    }

    pub fn empty() -> Self {
        Self(HashMap::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryInfoInternal(pub x25519::PublicKey, pub Vec<NetworkAddress>);

impl DiscoveryInfoInternal {
    pub fn try_from_validator_info(
        role_filter: RoleType,
        validator_info: ValidatorInfo,
    ) -> Result<Self> {
        let info = match role_filter {
            RoleType::Validator => Self(
                validator_info
                    .config()
                    .validator_network_identity_public_key,
                vec![NetworkAddress::try_from(
                    &validator_info.config().validator_network_address,
                )?],
            ),
            RoleType::FullNode => Self(
                validator_info
                    .config()
                    .full_node_network_identity_public_key,
                vec![NetworkAddress::try_from(
                    &validator_info.config().full_node_network_address,
                )?],
            ),
        };
        Ok(info)
    }
}

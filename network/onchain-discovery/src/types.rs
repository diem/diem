// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{bail, ensure, Error, Result};
use libra_config::config::RoleType;
use libra_crypto::x25519;
use libra_logger::prelude::*;
use libra_network_address::NetworkAddress;
use libra_types::{
    discovery_info::DiscoveryInfo,
    discovery_set::{
        DiscoverySet, DiscoverySetChangeEvent, GLOBAL_DISCOVERY_SET_CHANGE_EVENT_PATH,
    },
    get_with_proof::{
        RequestItem, ResponseItem, UpdateToLatestLedgerRequest, UpdateToLatestLedgerResponse,
    },
    PeerId,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom};

/// OnchainDiscovery LibraNet message types. These are sent over-the-wire.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum OnchainDiscoveryMsg {
    QueryDiscoverySetRequest(QueryDiscoverySetRequest),
    QueryDiscoverySetResponse(QueryDiscoverySetResponse),
}

/// A request for another peer's latest discovery set and (if needed) a validator
/// change proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryDiscoverySetRequest {
    pub client_known_version: u64,
    // TODO(philiphayes): actually use this sequence number.
    pub client_known_seq_num: u64,
}

impl Into<UpdateToLatestLedgerRequest> for QueryDiscoverySetRequest {
    fn into(self) -> UpdateToLatestLedgerRequest {
        let req_items = vec![RequestItem::GetEventsByEventAccessPath {
            access_path: GLOBAL_DISCOVERY_SET_CHANGE_EVENT_PATH.clone(),
            // u64::MAX means query the latest event
            start_event_seq_num: ::std::u64::MAX,
            // to query latest, ascending needs to be false
            ascending: false,
            // just query the last event
            limit: 1,
        }];

        UpdateToLatestLedgerRequest::new(self.client_known_version, req_items)
    }
}

/// A response to a QueryDiscoverySetRequest with the latest discovery set change
/// event and (if needed) a epoch change proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryDiscoverySetResponse {
    pub update_to_latest_ledger_response: Box<UpdateToLatestLedgerResponse>,
}

impl From<UpdateToLatestLedgerResponse> for QueryDiscoverySetResponse {
    fn from(res: UpdateToLatestLedgerResponse) -> Self {
        Self {
            update_to_latest_ledger_response: Box::new(res),
        }
    }
}

impl From<QueryDiscoverySetResponseWithEvent> for QueryDiscoverySetResponse {
    fn from(res: QueryDiscoverySetResponseWithEvent) -> Self {
        Self {
            update_to_latest_ledger_response: res.query_res.update_to_latest_ledger_response,
        }
    }
}

/// A QueryDiscoverySetResponse with the event validated, deserialized, and pulled
/// out to reduce unnecessary double validation.
#[derive(Clone, Debug)]
pub struct QueryDiscoverySetResponseWithEvent {
    pub query_res: QueryDiscoverySetResponse,
    pub event: Option<DiscoverySetChangeEvent>,
}

impl TryFrom<QueryDiscoverySetResponse> for QueryDiscoverySetResponseWithEvent {
    type Error = Error;

    fn try_from(query_res: QueryDiscoverySetResponse) -> Result<Self> {
        // TODO(philiphayes): validate event access path etc

        let res = &query_res.update_to_latest_ledger_response;
        let res_len = res.response_items.len();
        ensure!(
            res_len <= 1,
            "Should only contain at most one response item. response_items.len(): {}",
            res_len
        );

        let maybe_event = res.response_items.get(0)
            .map(|res_item| {
                match res_item {
                    ResponseItem::GetEventsByEventAccessPath {
                        events_with_proof,
                        ..
                    } => {
                        let events_len = events_with_proof.len();
                        ensure!(
                            events_len <= 1,
                            "Should only contain at most one event response. events_with_proof.len(): {}",
                            events_len
                        );

                        Ok(events_with_proof.get(0).map(|event_with_proof| &event_with_proof.event))
                    },
                    res_item => bail!("Unexpected response item type: res_item: {:?}, expected GetEventsByEventAccessPath", res_item),
                }
            })
            .transpose()?
            .flatten();

        let event = maybe_event
            .map(DiscoverySetChangeEvent::try_from)
            .transpose()?;

        Ok(Self { query_res, event })
    }
}

/// An internal representation of the DiscoverySet.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoverySetInternal(pub HashMap<PeerId, DiscoveryInfoInternal>);

impl DiscoverySetInternal {
    pub fn from_discovery_set(role_filter: RoleType, discovery_set: DiscoverySet) -> Self {
        Self(
            discovery_set
                .into_iter()
                .filter_map(|discovery_info| {
                    let peer_id = discovery_info.account_address;
                    let res_info =
                        DiscoveryInfoInternal::try_from_discovery_info(role_filter, discovery_info);

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
    pub fn try_from_discovery_info(
        role_filter: RoleType,
        discovery_info: DiscoveryInfo,
    ) -> Result<Self> {
        let info = match role_filter {
            RoleType::Validator => Self(
                discovery_info.validator_network_identity_pubkey,
                vec![NetworkAddress::try_from(
                    &discovery_info.validator_network_address,
                )?],
            ),
            RoleType::FullNode => Self(
                discovery_info.fullnodes_network_identity_pubkey,
                vec![NetworkAddress::try_from(
                    &discovery_info.fullnodes_network_address,
                )?],
            ),
        };
        Ok(info)
    }
}

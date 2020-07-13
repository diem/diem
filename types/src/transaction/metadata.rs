// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MetadataType {
    Undefined,
    GeneralMetadataType(GeneralMetadata),
    TravelRuleMetadataType(TravelRuleMetadata),
    UnstructuredStringMetadataType(UnstructuredStringMetadata),
}

// Used for versioning of general metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum GeneralMetadata {
    GeneralMetadataVersion0(GeneralMetadataV0),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct GeneralMetadataV0 {
    // Subaddress to which the funds are being sent
    #[serde(with = "serde_bytes")]
    to_subaddress: Option<Vec<u8>>,
    // Subaddress from which the funds are being sent
    #[serde(with = "serde_bytes")]
    from_subaddress: Option<Vec<u8>>,
    // Event sequence number of referenced payment
    referenced_event: Option<u64>,
}

// Used for versioning of travel rule metadata
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TravelRuleMetadata {
    TravelRuleMetadataVersion0(TravelRuleMetadataV0),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TravelRuleMetadataV0 {
    // Off-chain reference_id.  Used when off-chain APIs are used.
    // Specifies the off-chain reference ID that was agreed upon in off-chain APIs.
    off_chain_reference_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct UnstructuredStringMetadata {
    // Unstructured string metadata
    metadata: Option<String>,
}

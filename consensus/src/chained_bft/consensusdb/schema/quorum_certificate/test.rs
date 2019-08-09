// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use nextgen_crypto::ed25519::*;
use schemadb::schema::assert_encode_decode;

#[test]
fn test_encode_decode() {
    let qc = QuorumCert::certificate_for_genesis();
    assert_encode_decode::<QCSchema<Ed25519Signature>>(&qc.certified_block_id(), &qc);
}

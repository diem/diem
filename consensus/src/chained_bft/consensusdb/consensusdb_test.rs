// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use libra_tools::tempdir::TempPath;

#[test]
fn test_put_get() {
    let tmp_dir = TempPath::new();
    let db = ConsensusDB::new(&tmp_dir);

    let block = Block::<i64>::make_genesis_block();
    let blocks = vec![block];

    let old_blocks = db.get_blocks::<i64>().unwrap();
    assert!(db.get_state().unwrap().is_none());
    assert_eq!(old_blocks.len(), 0);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 0);

    // The second parameter here is a serialization of the last vote message, which is hard to
    // construct in this test, so it remains bogus (cannot be properly deserialized).
    db.save_state(vec![0x01, 0x02, 0x03], vec![0x01, 0x02, 0x03])
        .unwrap();

    let qcs = vec![QuorumCert::certificate_for_genesis()];

    db.save_blocks_and_quorum_certificates(blocks, qcs).unwrap();

    assert_eq!(db.get_blocks::<i64>().unwrap().len(), 1);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 1);
    assert!(!db.get_state().unwrap().is_none());
}

#[test]
fn test_delete_block_and_qc() {
    let tmp_dir = TempPath::new();
    let db = ConsensusDB::new(&tmp_dir);

    assert!(db.get_state().unwrap().is_none());
    assert_eq!(db.get_blocks::<i64>().unwrap().len(), 0);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 0);

    let blocks = vec![Block::<i64>::make_genesis_block()];
    let block_id = blocks[0].id();

    let qcs = vec![QuorumCert::certificate_for_genesis()];
    let qc_id = qcs[0].certified_block().id();

    db.save_blocks_and_quorum_certificates(blocks, qcs).unwrap();
    assert_eq!(db.get_blocks::<i64>().unwrap().len(), 1);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 1);

    // Start to delete
    db.delete_blocks_and_quorum_certificates::<i64>(vec![block_id, qc_id])
        .unwrap();
    assert_eq!(db.get_blocks::<i64>().unwrap().len(), 0);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 0);
}

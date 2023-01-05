// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

use super::*;
use consensus_types::block::block_test_utils::certificate_for_genesis;
use diem_temppath::TempPath;

#[test]
fn test_put_get() {
    let tmp_dir = TempPath::new();
    let db = ConsensusDB::new(&tmp_dir);

    let block = Block::make_genesis_block();
    let blocks = vec![block];

    assert_eq!(db.get_blocks().unwrap().len(), 0);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 0);

    let qcs = vec![certificate_for_genesis()];
    db.save_blocks_and_quorum_certificates(blocks.clone(), qcs.clone())
        .unwrap();

    assert_eq!(db.get_blocks().unwrap().len(), 1);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 1);

    let tc = vec![0u8, 1, 2];
    db.save_highest_timeout_certificate(tc.clone()).unwrap();
    db.save_highest_2chain_timeout_certificate(tc.clone())
        .unwrap();

    let vote = vec![2u8, 1, 0];
    db.save_vote(vote.clone()).unwrap();

    let (vote_1, tc_1, tc_2, blocks_1, qc_1) = db.get_data().unwrap();
    assert_eq!(blocks, blocks_1);
    assert_eq!(qcs, qc_1);
    assert_eq!(Some(tc.clone()), tc_1);
    assert_eq!(Some(tc), tc_2);
    assert_eq!(Some(vote), vote_1);

    db.delete_highest_timeout_certificate().unwrap();
    db.delete_highest_2chain_timeout_certificate().unwrap();
    db.delete_last_vote_msg().unwrap();
    assert!(db.get_highest_timeout_certificate().unwrap().is_none());
    assert!(db
        .get_highest_2chain_timeout_certificate()
        .unwrap()
        .is_none());
    assert!(db.get_last_vote().unwrap().is_none());
}

#[test]
fn test_delete_block_and_qc() {
    let tmp_dir = TempPath::new();
    let db = ConsensusDB::new(&tmp_dir);

    assert_eq!(db.get_blocks().unwrap().len(), 0);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 0);

    let blocks = vec![Block::make_genesis_block()];
    let block_id = blocks[0].id();

    let qcs = vec![certificate_for_genesis()];
    let qc_id = qcs[0].certified_block().id();

    db.save_blocks_and_quorum_certificates(blocks, qcs).unwrap();
    assert_eq!(db.get_blocks().unwrap().len(), 1);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 1);

    // Start to delete
    db.delete_blocks_and_quorum_certificates(vec![block_id, qc_id])
        .unwrap();
    assert_eq!(db.get_blocks().unwrap().len(), 0);
    assert_eq!(db.get_quorum_certificates().unwrap().len(), 0);
}

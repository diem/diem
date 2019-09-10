// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    account_state_blob::AccountStateBlob,
    ledger_info::LedgerInfo,
    proof::{
        verify_account_state, verify_event, verify_signed_transaction,
        verify_sparse_merkle_element, verify_test_accumulator_element, AccountStateProof,
        AccumulatorProof, EventAccumulatorInternalNode, EventProof, MerkleTreeInternalNode,
        SignedTransactionProof, SparseMerkleInternalNode, SparseMerkleLeafNode, SparseMerkleProof,
        TestAccumulatorInternalNode, TransactionAccumulatorInternalNode,
    },
    transaction::{
        Program, RawTransaction, SignedTransaction, TransactionInfo, TransactionListWithProof,
    },
    vm_error::StatusCode,
};
use crypto::{
    ed25519::*,
    hash::{
        CryptoHash, TestOnlyHash, TransactionAccumulatorHasher, ACCUMULATOR_PLACEHOLDER_HASH,
        GENESIS_BLOCK_ID, SPARSE_MERKLE_PLACEHOLDER_HASH,
    },
    HashValue,
};
use proptest::{collection::vec, prelude::*};

#[test]
fn test_verify_empty_accumulator() {
    let element_hash = b"hello".test_only_hash();
    let root_hash = *ACCUMULATOR_PLACEHOLDER_HASH;
    let proof = AccumulatorProof::new(vec![]);
    assert!(verify_test_accumulator_element(root_hash, element_hash, 0, &proof).is_err());
}

#[test]
fn test_verify_single_element_accumulator() {
    let element_hash = b"hello".test_only_hash();
    let root_hash = element_hash;
    let proof = AccumulatorProof::new(vec![]);
    assert!(verify_test_accumulator_element(root_hash, element_hash, 0, &proof).is_ok());
}

#[test]
fn test_verify_two_element_accumulator() {
    let element0_hash = b"hello".test_only_hash();
    let element1_hash = b"world".test_only_hash();
    let root_hash = TestAccumulatorInternalNode::new(element0_hash, element1_hash).hash();

    assert!(verify_test_accumulator_element(
        root_hash,
        element0_hash,
        0,
        &AccumulatorProof::new(vec![element1_hash]),
    )
    .is_ok());
    assert!(verify_test_accumulator_element(
        root_hash,
        element1_hash,
        1,
        &AccumulatorProof::new(vec![element0_hash]),
    )
    .is_ok());
}

#[test]
fn test_verify_three_element_accumulator() {
    let element0_hash = b"hello".test_only_hash();
    let element1_hash = b"world".test_only_hash();
    let element2_hash = b"!".test_only_hash();
    let internal0_hash = TestAccumulatorInternalNode::new(element0_hash, element1_hash).hash();
    let internal1_hash =
        TestAccumulatorInternalNode::new(element2_hash, *ACCUMULATOR_PLACEHOLDER_HASH).hash();
    let root_hash = TestAccumulatorInternalNode::new(internal0_hash, internal1_hash).hash();

    assert!(verify_test_accumulator_element(
        root_hash,
        element0_hash,
        0,
        &AccumulatorProof::new(vec![internal1_hash, element1_hash]),
    )
    .is_ok());
    assert!(verify_test_accumulator_element(
        root_hash,
        element1_hash,
        1,
        &AccumulatorProof::new(vec![internal1_hash, element0_hash]),
    )
    .is_ok());
    assert!(verify_test_accumulator_element(
        root_hash,
        element2_hash,
        2,
        &AccumulatorProof::new(vec![internal0_hash, *ACCUMULATOR_PLACEHOLDER_HASH]),
    )
    .is_ok());
}

#[test]
fn test_accumulator_proof_63_siblings_leftmost() {
    let element_hash = b"hello".test_only_hash();
    let mut siblings = vec![];
    for i in 0..63 {
        siblings.push(HashValue::new([i; 32]));
    }
    let root_hash = siblings
        .iter()
        .rev()
        .fold(element_hash, |hash, sibling_hash| {
            TestAccumulatorInternalNode::new(hash, *sibling_hash).hash()
        });
    let proof = AccumulatorProof::new(siblings);

    assert!(verify_test_accumulator_element(root_hash, element_hash, 0, &proof).is_ok());
}

#[test]
fn test_accumulator_proof_63_siblings_rightmost() {
    let element_hash = b"hello".test_only_hash();
    let mut siblings = vec![];
    for i in 0..63 {
        siblings.push(HashValue::new([i; 32]));
    }
    let root_hash = siblings
        .iter()
        .rev()
        .fold(element_hash, |hash, sibling_hash| {
            TestAccumulatorInternalNode::new(*sibling_hash, hash).hash()
        });
    let leaf_index = (std::u64::MAX - 1) / 2;
    let proof = AccumulatorProof::new(siblings);

    assert!(verify_test_accumulator_element(root_hash, element_hash, leaf_index, &proof).is_ok());
}

#[test]
fn test_accumulator_proof_64_siblings() {
    let element_hash = b"hello".test_only_hash();
    let mut siblings = vec![];
    for i in 0..64 {
        siblings.push(HashValue::new([i; 32]));
    }
    let root_hash = siblings
        .iter()
        .rev()
        .fold(element_hash, |hash, sibling_hash| {
            TestAccumulatorInternalNode::new(hash, *sibling_hash).hash()
        });
    let proof = AccumulatorProof::new(siblings);

    assert!(verify_test_accumulator_element(root_hash, element_hash, 0, &proof).is_err());
}

#[test]
fn test_verify_empty_sparse_merkle() {
    let key = b"hello".test_only_hash();
    let blob = b"world".to_vec().into();
    let root_hash = *SPARSE_MERKLE_PLACEHOLDER_HASH;
    let proof = SparseMerkleProof::new(None, vec![]);

    // Trying to show that this key doesn't exist.
    assert!(verify_sparse_merkle_element(root_hash, key, &None, &proof).is_ok());
    // Trying to show that this key exists.
    assert!(verify_sparse_merkle_element(root_hash, key, &Some(blob), &proof).is_err());
}

#[test]
fn test_verify_single_element_sparse_merkle() {
    let key = b"hello".test_only_hash();
    let blob: Option<AccountStateBlob> = Some((b"world".to_vec()).into());
    let blob_hash = blob.as_ref().unwrap().hash();
    let non_existing_blob = b"world?".to_vec().into();
    let root_hash = SparseMerkleLeafNode::new(key, blob_hash).hash();
    let proof = SparseMerkleProof::new(Some((key, blob_hash)), vec![]);

    // Trying to show this exact key exists with its value.
    assert!(verify_sparse_merkle_element(root_hash, key, &blob, &proof).is_ok());
    // Trying to show this exact key exists with another value.
    assert!(
        verify_sparse_merkle_element(root_hash, key, &Some(non_existing_blob), &proof).is_err()
    );
    // Trying to show this key doesn't exist.
    assert!(verify_sparse_merkle_element(root_hash, key, &None, &proof).is_err());

    let non_existing_key = b"HELLO".test_only_hash();

    // The proof can be used to show non_existing_key doesn't exist.
    assert!(verify_sparse_merkle_element(root_hash, non_existing_key, &None, &proof).is_ok());
    // The proof can't be used to non_existing_key exists.
    assert!(verify_sparse_merkle_element(root_hash, non_existing_key, &blob, &proof).is_err());
}

#[test]
fn test_verify_three_element_sparse_merkle() {
    //            root
    //           /    \
    //          a      default
    //         / \
    //     key1   b
    //           / \
    //       key2   key3
    let key1 = b"hello".test_only_hash();
    let key2 = b"world".test_only_hash();
    let key3 = b"!".test_only_hash();
    assert_eq!(key1[0], 0b0011_0011);
    assert_eq!(key2[0], 0b0100_0010);
    assert_eq!(key3[0], 0b0110_1001);

    let blob1 = Some(AccountStateBlob::from(b"1".to_vec()));
    let blob2 = Some(AccountStateBlob::from(b"2".to_vec()));
    let blob3 = Some(AccountStateBlob::from(b"3".to_vec()));

    let leaf1_hash = SparseMerkleLeafNode::new(key1, blob1.as_ref().unwrap().hash()).hash();
    let leaf2_hash = SparseMerkleLeafNode::new(key2, blob2.as_ref().unwrap().hash()).hash();
    let leaf3_hash = SparseMerkleLeafNode::new(key3, blob3.as_ref().unwrap().hash()).hash();
    let internal_b_hash = SparseMerkleInternalNode::new(leaf2_hash, leaf3_hash).hash();
    let internal_a_hash = SparseMerkleInternalNode::new(leaf1_hash, internal_b_hash).hash();
    let root_hash =
        SparseMerkleInternalNode::new(internal_a_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH).hash();

    let non_existing_key1 = b"abc".test_only_hash();
    let non_existing_key2 = b"def".test_only_hash();
    assert_eq!(non_existing_key1[0], 0b0011_1010);
    assert_eq!(non_existing_key2[0], 0b1000_1110);

    {
        // Construct a proof of key1.
        let proof = SparseMerkleProof::new(
            Some((key1, blob1.as_ref().unwrap().hash())),
            vec![*SPARSE_MERKLE_PLACEHOLDER_HASH, internal_b_hash],
        );

        // The exact key value exists.
        assert!(verify_sparse_merkle_element(root_hash, key1, &(blob1), &proof).is_ok());
        // Trying to show that this key has another value.
        assert!(verify_sparse_merkle_element(root_hash, key1, &(blob2), &proof).is_err());
        // Trying to show that this key doesn't exist.
        assert!(verify_sparse_merkle_element(root_hash, key1, &None, &proof).is_err());
        // This proof can't be used to show anything about key2.
        assert!(verify_sparse_merkle_element(root_hash, key2, &None, &proof).is_err());
        assert!(verify_sparse_merkle_element(root_hash, key2, &(blob1), &proof).is_err());
        assert!(verify_sparse_merkle_element(root_hash, key2, &(blob2), &proof).is_err());

        // This proof can be used to show that non_existing_key1 indeed doesn't exist.
        assert!(verify_sparse_merkle_element(root_hash, non_existing_key1, &None, &proof).is_ok());
        // This proof can't be used to show that non_existing_key2 doesn't exist because it lives
        // in a different subtree.
        assert!(verify_sparse_merkle_element(root_hash, non_existing_key2, &None, &proof).is_err());
    }

    {
        // Construct a proof of the default node.
        let proof = SparseMerkleProof::new(None, vec![internal_a_hash]);

        // This proof can't be used to show that a key starting with 0 doesn't exist.
        assert!(verify_sparse_merkle_element(root_hash, non_existing_key1, &None, &proof).is_err());
        // This proof can be used to show that a key starting with 1 doesn't exist.
        assert!(verify_sparse_merkle_element(root_hash, non_existing_key2, &None, &proof).is_ok());
    }
}

#[test]
fn test_verify_signed_transaction() {
    //            root
    //           /     \
    //         /         \
    //       a             b
    //      / \           / \
    //  txn0   txn1   txn2   default
    let txn_info0_hash = b"hello".test_only_hash();
    let txn_info2_hash = b"!".test_only_hash();

    let txn1_hash = HashValue::random();
    let state_root1_hash = b"a".test_only_hash();
    let event_root1_hash = b"b".test_only_hash();
    let txn_info1 = TransactionInfo::new(
        txn1_hash,
        state_root1_hash,
        event_root1_hash,
        /* gas_used = */ 0,
        /* major_status = */ StatusCode::EXECUTED,
    );
    let txn_info1_hash = txn_info1.hash();

    let internal_a_hash =
        TransactionAccumulatorInternalNode::new(txn_info0_hash, txn_info1_hash).hash();
    let internal_b_hash =
        TransactionAccumulatorInternalNode::new(txn_info2_hash, *ACCUMULATOR_PLACEHOLDER_HASH)
            .hash();
    let root_hash =
        TransactionAccumulatorInternalNode::new(internal_a_hash, internal_b_hash).hash();
    let consensus_data_hash = b"c".test_only_hash();
    let ledger_info = LedgerInfo::new(
        /* version = */ 2,
        root_hash,
        consensus_data_hash,
        *GENESIS_BLOCK_ID,
        0,
        /* timestamp = */ 10000,
        None,
    );

    let ledger_info_to_transaction_info_proof =
        AccumulatorProof::new(vec![internal_b_hash, txn_info0_hash]);
    let proof = SignedTransactionProof::new(ledger_info_to_transaction_info_proof, txn_info1);

    // The proof can be used to verify txn1.
    assert!(verify_signed_transaction(&ledger_info, txn1_hash, None, 1, &proof).is_ok());
    // Replacing txn1 with some other txn should cause the verification to fail.
    assert!(verify_signed_transaction(&ledger_info, HashValue::random(), None, 1, &proof).is_err());
    // Trying to show that txn1 is at version 2.
    assert!(verify_signed_transaction(&ledger_info, txn1_hash, None, 2, &proof).is_err());
}

#[test]
fn test_verify_account_state_and_event() {
    //                  root
    //                 /     \
    //               /         \
    //             a             b
    //            / \           / \
    //        txn0   txn1   txn2   default
    //                       ^
    //                       |
    //                 transaction_info2
    //                /    /           \
    //              /     /              \
    //    signed_txn  state_root          event_root
    //                  /    \               / \
    //                 c      default  event0   event1
    //                / \
    //            key1   d
    //                  / \
    //              key2   key3
    let key1 = b"hello".test_only_hash();
    let key2 = b"world".test_only_hash();
    let key3 = b"!".test_only_hash();
    let non_existing_key = b"#".test_only_hash();
    assert_eq!(key1[0], 0b0011_0011);
    assert_eq!(key2[0], 0b0100_0010);
    assert_eq!(key3[0], 0b0110_1001);
    assert_eq!(non_existing_key[0], 0b0100_0001);

    let blob1 = AccountStateBlob::from(b"value1".to_vec());
    let blob2 = AccountStateBlob::from(b"value2".to_vec());
    let blob3 = AccountStateBlob::from(b"value3".to_vec());

    let leaf1_hash = SparseMerkleLeafNode::new(key1, blob1.hash()).hash();
    let leaf2_hash = SparseMerkleLeafNode::new(key2, blob2.hash()).hash();
    let leaf3_hash = SparseMerkleLeafNode::new(key3, blob3.hash()).hash();
    let internal_d_hash = SparseMerkleInternalNode::new(leaf2_hash, leaf3_hash).hash();
    let internal_c_hash = SparseMerkleInternalNode::new(leaf1_hash, internal_d_hash).hash();
    let state_root_hash =
        SparseMerkleInternalNode::new(internal_c_hash, *SPARSE_MERKLE_PLACEHOLDER_HASH).hash();

    let txn_info0_hash = b"hellohello".test_only_hash();
    let txn_info1_hash = b"worldworld".test_only_hash();

    let (privkey, pubkey) = compat::generate_keypair(None);
    let txn2_hash = RawTransaction::new(
        AccountAddress::from_public_key(&pubkey),
        /* sequence_number = */ 0,
        Program::new(vec![], vec![], vec![]),
        /* max_gas_amount = */ 0,
        /* gas_unit_price = */ 0,
        /* expiration_time = */ std::time::Duration::new(0, 0),
    )
    .sign(&privkey, pubkey)
    .expect("Signing failed.")
    .hash();

    let event0_hash = b"event0".test_only_hash();
    let event1_hash = b"event1".test_only_hash();
    let event_root_hash = EventAccumulatorInternalNode::new(event0_hash, event1_hash).hash();

    let txn_info2 = TransactionInfo::new(
        txn2_hash,
        state_root_hash,
        event_root_hash,
        /* gas_used = */ 0,
        /* major_status = */ StatusCode::EXECUTED,
    );
    let txn_info2_hash = txn_info2.hash();

    let internal_a_hash =
        TransactionAccumulatorInternalNode::new(txn_info0_hash, txn_info1_hash).hash();
    let internal_b_hash =
        TransactionAccumulatorInternalNode::new(txn_info2_hash, *ACCUMULATOR_PLACEHOLDER_HASH)
            .hash();
    let root_hash =
        TransactionAccumulatorInternalNode::new(internal_a_hash, internal_b_hash).hash();

    // consensus_data_hash isn't used in proofs, but we need it to construct LedgerInfo.
    let consensus_data_hash = b"consensus_data".test_only_hash();
    let ledger_info = LedgerInfo::new(
        /* version = */ 2,
        root_hash,
        consensus_data_hash,
        *GENESIS_BLOCK_ID,
        0,
        /* timestamp = */ 10000,
        None,
    );

    let ledger_info_to_transaction_info_proof =
        AccumulatorProof::new(vec![internal_a_hash, *ACCUMULATOR_PLACEHOLDER_HASH]);
    let transaction_info_to_account_proof = SparseMerkleProof::new(
        Some((key2, blob2.hash())),
        vec![*SPARSE_MERKLE_PLACEHOLDER_HASH, leaf1_hash, leaf3_hash],
    );
    let account_state_proof = AccountStateProof::new(
        ledger_info_to_transaction_info_proof.clone(),
        txn_info2.clone(),
        transaction_info_to_account_proof,
    );

    // Prove that account at `key2` has value `value2`.
    assert!(verify_account_state(
        &ledger_info,
        /* state_version = */ 2,
        key2,
        &Some(blob2),
        &account_state_proof,
    )
    .is_ok());
    // Use the same proof to prove that `non_existing_key` doesn't exist.
    assert!(verify_account_state(
        &ledger_info,
        /* state_version = */ 2,
        non_existing_key,
        &None,
        &account_state_proof,
    )
    .is_ok());

    let bad_blob2 = b"3".to_vec().into();
    assert!(verify_account_state(
        &ledger_info,
        /* state_version = */ 2,
        key2,
        &Some(bad_blob2),
        &account_state_proof,
    )
    .is_err());

    let transaction_info_to_event_proof = AccumulatorProof::new(vec![event1_hash]);
    let event_proof = EventProof::new(
        ledger_info_to_transaction_info_proof.clone(),
        txn_info2.clone(),
        transaction_info_to_event_proof,
    );

    // Prove that the first event within transaction 2 is `event0`.
    assert!(verify_event(
        &ledger_info,
        event0_hash,
        /* transaction_version = */ 2,
        /* event_version_within_transaction = */ 0,
        &event_proof,
    )
    .is_ok());

    let bad_event0_hash = b"event1".test_only_hash();
    assert!(verify_event(
        &ledger_info,
        bad_event0_hash,
        /* transaction_version = */ 2,
        /* event_version_within_transaction = */ 0,
        &event_proof,
    )
    .is_err());
}

// Return a variable length of transaction_and_info list with a random range within [0,
// list_length).
fn arb_signed_txn_list_and_range(
) -> impl Strategy<Value = (Vec<(SignedTransaction, TransactionInfo)>, usize, usize)> {
    vec(
        (any::<SignedTransaction>(), any::<TransactionInfo>()),
        0..100,
    )
    .prop_flat_map(|list| {
        let len = list.len();
        (Just(list), 0..std::cmp::max(len, 1))
    })
    .prop_flat_map(|(list, start)| {
        let len = list.len();
        (Just(list), Just(start), start..std::cmp::max(len, 1))
    })
    .prop_map(|(list, start, end)| {
        let final_list = list
            .into_iter()
            .map(|(txn, txn_info)| {
                let txn_hash = txn.hash();
                (
                    txn,
                    TransactionInfo::new(
                        txn_hash,
                        txn_info.state_root_hash(),
                        txn_info.event_root_hash(),
                        txn_info.gas_used(),
                        txn_info.major_status(),
                    ),
                )
            })
            .collect::<Vec<_>>();
        (final_list, start, end)
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    #[test]
    fn test_transaction_list_with_proof((txn_and_infos, first_version, last_version) in arb_signed_txn_list_and_range()) {
        let mut root_hash = *ACCUMULATOR_PLACEHOLDER_HASH;

        let txn_list_with_proof =
           if txn_and_infos.is_empty() {
               TransactionListWithProof::new(vec![], None, None, None, None)
           } else {
               let mut hashes = txn_and_infos
                   .iter()
                   .map(|(_, txn_info)|
                       txn_info.hash()
                   ).collect::<Vec<_>>();
               if hashes.len() % 2 == 1 && hashes.len() != 1 {
                   hashes.push(*ACCUMULATOR_PLACEHOLDER_HASH);
               }
               let mut tree = vec![hashes];
               while tree.last().unwrap().len() > 1 {
                   let mut parent_hashes = vec![];
                   let mut hash_iter = tree.last().unwrap().iter();
                   while let Some(left) = hash_iter.next() {
                       let right = hash_iter.next().expect("Can't be None");
                       parent_hashes.push(
                           MerkleTreeInternalNode::<TransactionAccumulatorHasher>::new(*left, *right).hash(),
                       )
                   }
                   hashes = parent_hashes;
                   if hashes.len() % 2 == 1 && hashes.len() != 1 {
                       hashes.push(*ACCUMULATOR_PLACEHOLDER_HASH);
                   }
                   tree.push(hashes);
               }
               assert_eq!(tree.last().unwrap().len(), 1);
               root_hash = tree.pop().unwrap()[0];

               // Get proofs.
               let mut first_index = first_version;
               let mut last_index = last_version;
               let mut first_siblings = vec![];
               let mut last_siblings = vec![];
               for nodes in tree {
                   first_siblings.push(
                       if first_index % 2 == 0 {
                           nodes[first_index + 1]
                       } else {
                           nodes[first_index - 1]
                       }
                   );
                   last_siblings.push(
                       if last_index % 2 == 0 {
                           nodes[last_index + 1]
                       } else {
                           nodes[last_index - 1]
                       }
                   );
                   first_index /= 2;
                   last_index /= 2;
               }
               let first_proof =
                   Some(AccumulatorProof::new(first_siblings.into_iter().rev().collect::<Vec<_>>()));
               let last_proof = if first_version == last_version {
                   None
               } else {
                   Some(AccumulatorProof::new(last_siblings.into_iter().rev().collect::<Vec<_>>()))
               };

               TransactionListWithProof::new(
                   txn_and_infos[first_version..=last_version].to_vec(),
                   None,
                   Some(first_version as u64),
                   first_proof,
                   last_proof,
               )
           };

        // consensus_data_hash isn't used in proofs, but we need it to construct LedgerInfo.
        let consensus_data_hash = b"consensus_data".test_only_hash();
        let ledger_info = LedgerInfo::new(
            /* version = */ std::cmp::max(1, txn_and_infos.len()) as u64 - 1,
            root_hash,
            consensus_data_hash,
            *GENESIS_BLOCK_ID,
            0,
            /* timestamp = */ 10000,
            None,
        );
        let first_version = if txn_and_infos.is_empty() { None } else { Some(first_version as u64) };
        prop_assert!(txn_list_with_proof.verify(&ledger_info,first_version).is_ok());
    }
}

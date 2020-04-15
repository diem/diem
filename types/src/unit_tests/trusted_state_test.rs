// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::{
    account_address::AccountAddress,
    block_info::BlockInfo,
    ledger_info::{LedgerInfo, LedgerInfoWithSignatures},
    on_chain_config::ValidatorSet,
    transaction::Version,
    trusted_state::{TrustedState, TrustedStateChange},
    validator_change::ValidatorChangeProof,
    validator_info::ValidatorInfo,
    validator_signer::ValidatorSigner,
    validator_verifier::random_validator_verifier,
    waypoint::Waypoint,
};
use libra_crypto::{
    ed25519,
    hash::{CryptoHash, HashValue},
    test_utils::TEST_SEED,
};
use proptest::{
    collection::{size_range, vec, SizeRange},
    prelude::*,
    sample::Index,
};
use rand::prelude::*;
use std::collections::BTreeMap;

// hack strategy to generate a length from `impl Into<SizeRange>`
fn arb_length(size_range: impl Into<SizeRange>) -> impl Strategy<Value = usize> {
    vec(Just(()), size_range).prop_map(|vec| vec.len())
}

/// For `n` epoch changes, we sample `n+1` validator sets of variable size
/// `validators_per_epoch`. The `+1` is for the initial validator set in the first
/// epoch.
fn arb_validator_sets(
    epoch_changes: impl Into<SizeRange>,
    validators_per_epoch: impl Into<SizeRange>,
) -> impl Strategy<Value = Vec<Vec<ValidatorSigner>>> {
    vec(arb_length(validators_per_epoch), epoch_changes.into() + 1).prop_map(
        |validators_per_epoch_vec| {
            validators_per_epoch_vec
                .into_iter()
                .map(|num_validators| {
                    // all uniform voting power
                    let voting_power = None;
                    // human readable incrementing account addresses
                    let int_account_addrs = true;
                    let (signers, _verifier) =
                        random_validator_verifier(num_validators, voting_power, int_account_addrs);
                    signers
                })
                .collect::<Vec<_>>()
        },
    )
}

/// Convert a slice of `ValidatorSigner` (includes the private signing key) into
/// the public-facing `ValidatorSet` type (just the public key).
fn into_validator_set(signers: &[ValidatorSigner]) -> ValidatorSet {
    let mut rng = StdRng::from_seed(TEST_SEED);
    ValidatorSet::new(
        signers
            .iter()
            .map(|signer| {
                ValidatorInfo::new_with_test_network_keys(
                    &mut rng,
                    signer.author(),
                    signer.public_key(),
                    1, /* voting power */
                )
            })
            .collect::<Vec<_>>(),
    )
}

/// Create all signatures for a `LedgerInfoWithSignatures` given a set of signers
/// and a `LedgerInfo`.
fn sign_ledger_info(
    signers: &[ValidatorSigner],
    ledger_info: &LedgerInfo,
) -> BTreeMap<AccountAddress, ed25519::Signature> {
    signers
        .iter()
        .map(|s| (s.author(), s.sign_message(ledger_info.hash())))
        .collect()
}

fn new_mock_ledger_info(
    epoch: u64,
    version: Version,
    next_validator_set: Option<ValidatorSet>,
) -> LedgerInfo {
    LedgerInfo::new(
        BlockInfo::new(
            epoch,
            0,                 /* round */
            HashValue::zero(), /* id */
            HashValue::zero(), /* executed_state_id */
            version,
            0, /* timestamp_usecs */
            next_validator_set,
        ),
        HashValue::zero(),
    )
}

// A strategy for generating components of an UpdateToLatestLedgerResponse with
// a correct ValidatorChangeProof.
fn arb_update_proof(
    // the epoch of the first LedgerInfoWithSignatures
    start_epoch: u64,
    // the version of the first LedgerInfoWithSignatures
    start_version: Version,
    // the distribution of versions changes between LedgerInfoWithSignatures
    version_delta: impl Into<SizeRange>,
    // the distribution for the number of epoch changes to generate
    epoch_changes: impl Into<SizeRange>,
    // the distribution for the number of validators in each epoch
    validators_per_epoch: impl Into<SizeRange>,
) -> impl Strategy<
    Value = (
        // The validator sets for each epoch
        Vec<Vec<ValidatorSigner>>,
        // The epoch change ledger infos
        Vec<LedgerInfoWithSignatures>,
        // The latest ledger info inside the last epoch
        LedgerInfoWithSignatures,
    ),
> {
    // helpful diagram:
    //
    // input:
    //   num epoch changes
    //
    // output:
    //   vsets: [S_1 .. S_n+1],
    //   epoch changes: [L_1, .., L_n],
    //   latest ledger_info: L_n+1
    //
    // let S_i = ith set of validators
    // let L_i = ith ledger info
    // S_i -> L_i => ith validators sign ith ledger info
    // L_i -> S_i+1 => ith ledger info contains i+1'th validators for epoch change
    // L_n+1 = a ledger info inside the nth epoch (contains S = None)
    //
    // base case: n = 0 => no epoch changes
    //
    // [ S_1 ] (None)
    //     \   __^
    //      v /
    //    [ L_1 ]
    //
    // otherwise, for n > 0:
    //
    // [ S_1, S_2, ..., S_n+1 ] (None)
    //    \    ^ \       ^ \   __^
    //     v  /   v     /   v /
    //    [ L_1, L_2, ..., L_n+1 ]
    //

    let version_delta = size_range(version_delta);
    let epoch_changes = size_range(epoch_changes);
    let validators_per_epoch = size_range(validators_per_epoch);

    // sample n, the number of epoch changes
    arb_length(epoch_changes).prop_flat_map(move |epoch_changes| {
        (
            // sample the validator sets, including the signers for the first epoch
            arb_validator_sets(epoch_changes, validators_per_epoch.clone()),
            // generate n version deltas
            vec(arb_length(version_delta.clone()), epoch_changes),
        )
            .prop_map(move |(mut vsets, version_deltas)| {
                // if generating from genesis, then there is no validator set to
                // sign the genesis block.
                if start_epoch == 0 {
                    // this will always succeed, since
                    // n >= 0, |vsets| = n + 1 ==> |vsets| >= 1
                    let pre_genesis_vset = vsets.first_mut().unwrap();
                    *pre_genesis_vset = vec![];
                }

                let mut epoch = start_epoch;
                let mut version = start_version;
                let num_epoch_changes = vsets.len() - 1;

                let signers = vsets.iter().take(num_epoch_changes);
                let next_sets = vsets.iter().skip(1);

                let ledger_infos_with_sigs = signers
                    .zip(next_sets)
                    .zip(version_deltas)
                    .map(|((curr_vset, next_vset), version_delta)| {
                        let next_vset = into_validator_set(next_vset);
                        let ledger_info = new_mock_ledger_info(epoch, version, Some(next_vset));
                        let signatures = sign_ledger_info(&curr_vset[..], &ledger_info);

                        epoch += 1;
                        version += version_delta as u64;

                        LedgerInfoWithSignatures::new(ledger_info, signatures)
                    })
                    .collect::<Vec<_>>();

                // this will always succeed, since
                // n >= 0, |vsets| = n + 1 ==> |vsets| >= 1
                let last_vset = vsets.last().unwrap();
                let latest_ledger_info = new_mock_ledger_info(epoch, version, None);
                let signatures = sign_ledger_info(&last_vset[..], &latest_ledger_info);
                let latest_ledger_info_with_sigs =
                    LedgerInfoWithSignatures::new(latest_ledger_info, signatures);
                (vsets, ledger_infos_with_sigs, latest_ledger_info_with_sigs)
            })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn test_pre_genesis_trusted_state_always_ratchets_from_any_genesis(
        (_vsets, lis_with_sigs, latest_li) in arb_update_proof(
            0    /* start epoch */,
            0    /* start version */,
            1..3 /* version delta */,
            1..3 /* epoch changes */,
            1..5 /* validators per epoch */,
        )
    ) {
        let trusted_state = TrustedState::new_trust_any_genesis_WARNING_UNSAFE();

        let expected_latest_version = latest_li.ledger_info().version();
        let expected_latest_li = latest_li.clone();
        let expected_latest_epoch_change_li = lis_with_sigs.last().cloned();
        let expected_validator_set = expected_latest_epoch_change_li
            .as_ref()
            .and_then(|li_with_sigs| li_with_sigs.ledger_info().next_validator_set());

        let change_proof = ValidatorChangeProof::new(lis_with_sigs, false /* more */);
        let trusted_state_change = trusted_state
            .verify_and_ratchet(&latest_li, &change_proof)
            .expect("Should never error or be stale when ratcheting from pre-genesis with valid proofs");

        match trusted_state_change {
            TrustedStateChange::Epoch {
                new_state,
                latest_li,
                latest_epoch_change_li,
                latest_validator_set,
            } => {
                assert_eq!(new_state.latest_version(), expected_latest_version);
                assert_eq!(latest_li, &expected_latest_li);
                assert_eq!(Some(latest_epoch_change_li), expected_latest_epoch_change_li.as_ref());
                assert_eq!(Some(latest_validator_set), expected_validator_set);
            }
            TrustedStateChange::Version {
                new_state,
                latest_li,
            } => {
                panic!(
                    "Unexpected version-only change, expecting epoch change: \
                     new_state: {:?}, latest_li: {:?}",
                    new_state,
                    latest_li,
                );
            }
        };
    }

    #[test]
    fn test_ratchet_from_waypoint(
        (_vsets, lis_with_sigs, latest_li) in arb_update_proof(
            10,   /* start epoch */
            123,  /* start version */
            1..3, /* version delta */
            1..3, /* epoch changes */
            1..5, /* validators per epoch */
        )
    ) {
        let first_epoch_change_li = lis_with_sigs.first().map(|l| l.ledger_info()).unwrap();
        let waypoint = Waypoint::new(first_epoch_change_li)
            .expect("Generating waypoint failed even though we passed an epoch change ledger info");
        let trusted_state = TrustedState::from_waypoint(waypoint);

        let expected_latest_version = latest_li.ledger_info().version();
        let expected_latest_li = latest_li.clone();
        let expected_latest_epoch_change_li = lis_with_sigs.last().cloned();
        let expected_validator_set = expected_latest_epoch_change_li
            .as_ref()
            .and_then(|li_with_sigs| li_with_sigs.ledger_info().next_validator_set());

        let change_proof = ValidatorChangeProof::new(lis_with_sigs, false /* more */);
        let trusted_state_change = trusted_state
            .verify_and_ratchet(&latest_li, &change_proof)
            .expect("Should never error or be stale when ratcheting from waypoint with valid proofs");

        match trusted_state_change {
            TrustedStateChange::Epoch {
                new_state,
                latest_li,
                latest_epoch_change_li,
                latest_validator_set,
            } => {
                assert_eq!(new_state.latest_version(), expected_latest_version);
                assert_eq!(latest_li, &expected_latest_li);
                assert_eq!(Some(latest_epoch_change_li), expected_latest_epoch_change_li.as_ref());
                assert_eq!(Some(latest_validator_set), expected_validator_set);
            }
            TrustedStateChange::Version { .. } =>
                panic!("Ratcheting from a waypoint should always provide the epoch for that waypoint"),
        };
    }

    #[test]
    fn test_ratchet_version_only(
        (_vsets, mut lis_with_sigs, latest_li) in arb_update_proof(
            1,    /* start epoch */
            1,    /* start version */
            1..3, /* version delta */
            1,    /* epoch changes */
            1..5, /* validators per epoch */
        )
    ) {
        // Assume we have already ratcheted into this epoch
        let epoch_change_li = lis_with_sigs.remove(0);
        let trusted_state = TrustedState::from_epoch_change_ledger_info(
            epoch_change_li.ledger_info().version() + 1,
            epoch_change_li.ledger_info()
        ).unwrap();

        let expected_latest_version = latest_li.ledger_info().version();
        let expected_latest_li = latest_li.clone();

        // Use an empty epoch change proof
        let change_proof = ValidatorChangeProof::new(vec![], false /* more */);
        let trusted_state_change = trusted_state
            .verify_and_ratchet(&latest_li, &change_proof)
            .expect("Should never error or be stale when ratcheting from waypoint with valid proofs");

        match trusted_state_change {
            TrustedStateChange::Epoch { ..  } =>
                panic!("Empty change proof so we should not change epoch"),
            TrustedStateChange::Version {
                new_state,
                latest_li,
            } => {
                assert_eq!(new_state.latest_version(), expected_latest_version);
                assert_eq!(latest_li, &expected_latest_li);
            }
        };
    }

    #[test]
    fn test_ratchet_with_partial_trusted_prefix(
        (_vsets, lis_with_sigs, latest_li) in arb_update_proof(
            1,    /* start epoch */
            1,    /* start version */
            1..3, /* version delta */
            1..5, /* epoch changes */
            1..5, /* validators per epoch */
        ),
        trusted_prefix_end_idx in any::<Index>(),
    ) {
        // Let's say we ratcheted to an intermediate epoch concurrently while this
        // update request was fulfilled. If the response still has a fresher state,
        // we should be able to use that and just skip the already trusted prefix.
        let idx = trusted_prefix_end_idx.index(lis_with_sigs.len());
        let intermediate_ledger_info = lis_with_sigs[idx].ledger_info();
        let trusted_state = TrustedState::from_epoch_change_ledger_info(
            intermediate_ledger_info.version() + 1,
            intermediate_ledger_info,
        ).unwrap();

        let expected_latest_version = latest_li.ledger_info().version();
        let expected_latest_li = latest_li.clone();
        let expected_latest_epoch_change_li = lis_with_sigs.last().cloned();
        let expected_validator_set = expected_latest_epoch_change_li
            .as_ref()
            .and_then(|li_with_sigs| li_with_sigs.ledger_info().next_validator_set());

        let change_proof = ValidatorChangeProof::new(lis_with_sigs, false /* more */);
        let trusted_state_change = trusted_state
            .verify_and_ratchet(&latest_li, &change_proof)
            .expect("Should never error or be stale when ratcheting from waypoint with valid proofs");

        match trusted_state_change {
            TrustedStateChange::Epoch {
                new_state,
                latest_li,
                latest_epoch_change_li,
                latest_validator_set,
            } => {
                assert_eq!(new_state.latest_version(), expected_latest_version);
                assert_eq!(latest_li, &expected_latest_li);
                assert_eq!(Some(latest_epoch_change_li), expected_latest_epoch_change_li.as_ref());
                assert_eq!(Some(latest_validator_set), expected_validator_set);
            }
            TrustedStateChange::Version {
                new_state,
                latest_li,
            } => {
                assert_eq!(new_state.latest_version(), expected_latest_version);
                assert_eq!(latest_li, &expected_latest_li);
            }
        };
    }

    #[test]
    fn test_ratchet_fails_with_gap_in_proof(
        (_vsets, mut lis_with_sigs, latest_li) in arb_update_proof(
            1,    /* start epoch */
            1,    /* start version */
            3,    /* version delta */
            2..5, /* epoch changes */
            1..3, /* validators per epoch */
        ),
        li_gap_idx in any::<Index>(),
    ) {
        let initial_li_with_sigs = lis_with_sigs.remove(0);
        let initial_li = initial_li_with_sigs.ledger_info();
        let trusted_state = TrustedState::from_epoch_change_ledger_info(
            initial_li.version() + 1,
            initial_li,
        ).unwrap();

        // materialize index and remove an epoch change in the proof to add a gap
        let li_gap_idx = li_gap_idx.index(lis_with_sigs.len());
        lis_with_sigs.remove(li_gap_idx);

        let change_proof = ValidatorChangeProof::new(lis_with_sigs, false /* more */);
        trusted_state
            .verify_and_ratchet(&latest_li, &change_proof)
            .expect_err("Should always return Err with an invalid change proof");
    }

    #[test]
    fn test_ratchet_fails_with_invalid_signature(
        (_vsets, mut lis_with_sigs, latest_li) in arb_update_proof(
            1,    /* start epoch */
            1,    /* start version */
            1,    /* version delta */
            2..5, /* epoch changes */
            1..5, /* validators per epoch */
        ),
        bad_li_idx in any::<Index>(),
    ) {
        let initial_li_with_sigs = lis_with_sigs.remove(0);
        let initial_li = initial_li_with_sigs.ledger_info();
        let trusted_state = TrustedState::from_epoch_change_ledger_info(
            initial_li.version() + 1,
            initial_li,
        ).unwrap();

        // Swap in a bad ledger info without signatures
        let li_with_sigs = bad_li_idx.get(&lis_with_sigs);
        let bad_li_with_sigs = LedgerInfoWithSignatures::new(
            li_with_sigs.ledger_info().clone(),
            BTreeMap::new(), /* empty signatures */
        );
        ::std::mem::replace(bad_li_idx.get_mut(&mut lis_with_sigs), bad_li_with_sigs);

        let change_proof = ValidatorChangeProof::new(lis_with_sigs, false /* more */);
        trusted_state
            .verify_and_ratchet(&latest_li, &change_proof)
            .expect_err("Should always return Err with an invalid change proof");
    }

    #[test]
    fn test_ratchet_fails_with_latest_li_invalid_signature(
        (_vsets, mut lis_with_sigs, latest_li) in arb_update_proof(
            1,    /* start epoch */
            1,    /* start version */
            1,    /* version delta */
            1..5, /* epoch changes */
            1..5, /* validators per epoch */
        ),
    ) {
        let initial_li_with_sigs = lis_with_sigs.remove(0);
        let initial_li = initial_li_with_sigs.ledger_info();
        let trusted_state = TrustedState::from_epoch_change_ledger_info(
            initial_li.version() + 1,
            initial_li,
        ).unwrap();

        // Verifying a latest ledger info (inside the last epoch) with invalid
        // signatures should fail.
        let latest_li = LedgerInfoWithSignatures::new(
            latest_li.ledger_info().clone(),
            BTreeMap::new(), /* empty signatures */
        );

        let change_proof = ValidatorChangeProof::new(lis_with_sigs, false /* more */);
        trusted_state
            .verify_and_ratchet(&latest_li, &change_proof)
            .expect_err("Should always return Err with a invalid latest li sigs");
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1))]

    #[test]
    fn test_stale_ratchet(
        (_vsets, lis_with_sigs, latest_li) in arb_update_proof(
            1,    /* start epoch */
            1,    /* start version */
            1..3, /* version delta */
            1,    /* epoch changes */
            1..5, /* validators per epoch */
        ),
    ) {
        // We've ratched beyond the response change proof, so attempting to ratchet
        // that change proof should just return `TrustedStateChange::Stale`.
        let epoch_change_li = new_mock_ledger_info(123 /* epoch */, 456 /* version */, Some(ValidatorSet::empty()));
        let trusted_state = TrustedState::from_epoch_change_ledger_info(
            epoch_change_li.version() + 1,
            &epoch_change_li,
        ).unwrap();

        let change_proof = ValidatorChangeProof::new(lis_with_sigs, false /* more */);
        trusted_state
            .verify_and_ratchet(&latest_li, &change_proof)
            .expect_err("Expected stale change, got valid change");
    }
}

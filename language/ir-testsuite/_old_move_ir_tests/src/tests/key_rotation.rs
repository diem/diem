// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::*;
use language_common::{error_codes::*, tooling::fake_executor::Account};
use libra_types::account_address::AccountAddress;
use move_ir::{assert_error_type, assert_no_error};

#[test]
fn cant_send_transaction_with_old_key_after_rotation() {
    let mut test_env = TestEnvironment::default();
    // Not a public key anyone can sign for
    let new_key = "deadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeefdeadbeef";

    let program = format!(
        "
import 0x0.LibraAccount;
main() {{
  let new_key;
  new_key = b\"{}\";
  LibraAccount.rotate_authentication_key(move(new_key));

  return;
}}",
        new_key
    );

    // rotate key
    assert_no_error!(test_env.run(to_standalone_script(program.as_bytes())));

    // prologue will fail when signing with old key after rotation
    assert_error_type!(
        test_env.run(to_standalone_script(b"main() { return; }")),
        ErrorKind::AssertError(EBAD_ACCOUNT_AUTHENTICATION_KEY, _)
    )
}

#[test]
fn can_send_transaction_with_new_key_after_rotation() {
    let mut test_env = TestEnvironment::default();

    let (privkey, pubkey) = compat::generate_keypair(&mut test_env.accounts.randomness_source);
    let program = format!(
        "
import 0x0.LibraAccount;
main() {{
  let new_key;
  new_key = b\"{}\";
  LibraAccount.rotate_authentication_key(move(new_key));

  return;
}}",
        hex::encode(AccountAddress::from_public_key(pubkey))
    );

    // rotate key
    assert_no_error!(test_env.run(to_standalone_script(program.as_bytes())));

    // we need to use the new key in order to send a transaction
    let old_account = test_env.accounts.get_account(0);
    let new_account = Account {
        addr: old_account.addr,
        privkey,
        pubkey,
    };

    let sequence_number = test_env.get_txn_sequence_number(0);
    let txn = test_env.create_user_txn(
        to_standalone_script(b"main() { return; }"),
        old_account.addr,
        new_account,
        sequence_number,
        TestEnvironment::DEFAULT_MAX_GAS,
        TestEnvironment::DEFAULT_GAS_COST,
    );

    assert_no_error!(test_env.run_txn(txn))
}

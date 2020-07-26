//! account: bob, 0, 0, address
//! account: charlie, 0, 0, address

//! new-transaction
//! sender: libraroot
//! args: 1, {{bob}}, {{bob::auth_key}}, b"validator_bob"
stdlib_script::create_validator_account
//! check: EXECUTED

// replay fails

//! new-transaction
//! sender: libraroot
//! args: 1, {{bob}}, {{bob::auth_key}}, b"validator_bob"
stdlib_script::create_validator_account
// check: "Keep(ABORTED { code: 3,"

//! new-transaction
//! sender: libraroot
//! args: 2, {{charlie}}, {{charlie::auth_key}}, b"validator_charlie"
stdlib_script::create_validator_account
//! check: EXECUTED

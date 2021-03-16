//! account: bob, 0, 0, address
//! account: alice, 0, 0, address
//! account: child, 0, 0, address

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{bob}}, {{bob::auth_key}}, b"bob", true
stdlib_script::AccountCreationScripts::create_parent_vasp_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
//! type-args: 0x1::XUS::XUS
//! args: {{child}}, {{child::auth_key}}, true, 0
stdlib_script::AccountCreationScripts::create_child_vasp_account
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XDX::XDX
//! args: 0, {{alice}}, {{alice::auth_key}}, b"alice", true
stdlib_script::AccountCreationScripts::create_parent_vasp_account
// check: "Keep(EXECUTED)"

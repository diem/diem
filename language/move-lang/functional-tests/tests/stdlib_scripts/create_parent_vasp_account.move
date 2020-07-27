//! account: bob, 0, 0, address
//! account: child, 0, 0, address

//! new-transaction
//! sender: libraroot
//! type-args: 0x1::Coin1::Coin1
//! args: {{bob}}, {{bob::auth_key}}, b"bob", b"boburl", x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d", true
stdlib_script::create_parent_vasp_account
// check: EXECUTED

//! new-transaction
//! sender: bob
//! type-args: 0x1::Coin2::Coin2
//! args: {{child}}, {{child::auth_key}}, true, 0
stdlib_script::create_child_vasp_account
// check: EXECUTED

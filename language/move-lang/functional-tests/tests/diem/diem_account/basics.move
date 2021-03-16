//! account: bob, 10000XDX
//! account: alice, 0XDX
//! account: abby, 0, 0, address
//! account: doris, 0XUS, 0

module Holder {
    use 0x1::Signer;

    struct Hold<T> has key {
        x: T
    }

    public fun hold<T: store>(account: &signer, x: T) {
        move_to(account, Hold<T>{x})
    }

    public fun get<T: store>(account: &signer): T
    acquires Hold {
        let Hold {x} = move_from<Hold<T>>(Signer::address_of(account));
        x
    }
}

//! new-transaction
script {
    use 0x1::DiemAccount;
    fun main(sender: signer) {
        let sender = &sender;
        DiemAccount::initialize(sender, x"00000000000000000000000000000000");
    }
}
// check: "Keep(ABORTED { code: 1,"

//! new-transaction
//! sender: bob
script {
    use 0x1::XDX::XDX;
    use 0x1::DiemAccount;
    fun main(account: signer) {
        let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XDX>(&with_cap, {{bob}}, 10, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use 0x1::XDX::XDX;
    use 0x1::DiemAccount;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XDX>(&with_cap, {{abby}}, 10, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 4357,"

//! new-transaction
//! sender: bob
script {
    use 0x1::XUS::XUS;
    use 0x1::DiemAccount;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XUS>(&with_cap, {{abby}}, 10, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 4869,"

//! new-transaction
//! sender: bob
script {
    use 0x1::XDX::XDX;
    use 0x1::DiemAccount;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XDX>(&with_cap, {{doris}}, 10, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(ABORTED { code: 4615,"

//! new-transaction
//! sender: bob
script {
    use 0x1::DiemAccount;
    fun main(account: signer) {
    let account = &account;
        let rot_cap = DiemAccount::extract_key_rotation_capability(account);
        DiemAccount::rotate_authentication_key(&rot_cap, x"123abc");
        DiemAccount::restore_key_rotation_capability(rot_cap);
    }
}
// check: "Keep(ABORTED { code: 2055,"

//! new-transaction
script {
    use 0x1::DiemAccount;
    use {{default}}::Holder;
    fun main(account: signer) {
    let account = &account;
        Holder::hold(
            account,
            DiemAccount::extract_key_rotation_capability(account)
        );
        Holder::hold(
            account,
            DiemAccount::extract_key_rotation_capability(account)
        );
    }
}
// check: "Keep(ABORTED { code: 2305,"

//! new-transaction
script {
    use 0x1::DiemAccount;
    use 0x1::Signer;
    fun main(sender: signer) {
    let sender = &sender;
        let cap = DiemAccount::extract_key_rotation_capability(sender);
        assert(
            *DiemAccount::key_rotation_capability_address(&cap) == Signer::address_of(sender), 0
        );
        DiemAccount::restore_key_rotation_capability(cap);
        let with_cap = DiemAccount::extract_withdraw_capability(sender);

        assert(
            *DiemAccount::withdraw_capability_address(&with_cap) == Signer::address_of(sender),
            0
        );
        DiemAccount::restore_withdraw_capability(with_cap);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
    use 0x1::DiemAccount;
    use 0x1::XDX::XDX;
    fun main(account: signer) {
    let account = &account;
        let with_cap = DiemAccount::extract_withdraw_capability(account);
        DiemAccount::pay_from<XDX>(&with_cap, {{alice}}, 10000, x"", x"");
        DiemAccount::restore_withdraw_capability(with_cap);
        assert(DiemAccount::balance<XDX>({{alice}}) == 10000, 60)
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, 0x0, {{bob::auth_key}}, b"bob", true
stdlib_script::AccountCreationScripts::create_parent_vasp_account
// check: "Keep(ABORTED { code: 2567,"

//! new-transaction
//! sender: blessed
//! type-args: 0x1::XUS::XUS
//! args: 0, {{abby}}, x"", b"bob", true
stdlib_script::AccountCreationScripts::create_parent_vasp_account
// check: "Keep(ABORTED { code: 2055,"

//! new-transaction
script {
use 0x1::DiemAccount;
fun main() {
    DiemAccount::sequence_number(0x1);
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
script {
use 0x1::DiemAccount;
fun main() {
    DiemAccount::authentication_key(0x1);
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
script {
use 0x1::DiemAccount;
fun main() {
    DiemAccount::delegated_key_rotation_capability(0x1);
}
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
script {
use 0x1::DiemAccount;
fun main() {
    DiemAccount::delegated_withdraw_capability(0x1);
}
}
// check: "Keep(ABORTED { code: 5,"

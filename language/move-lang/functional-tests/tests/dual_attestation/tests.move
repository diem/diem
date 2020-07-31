//! account: freddy_mac
//! account: bob, 0, 0, address

//! new-transaction
//! sender: freddy_mac
script{
    use 0x1::DualAttestation;
    fun main() {
        DualAttestation::get_cur_microlibra_limit();
    }
}
// check: EXECUTED

//! new-transaction
script{
    use 0x1::DualAttestation::{Self};
    fun main(not_blessed: &signer) {
        DualAttestation::set_microlibra_limit(not_blessed, 99);
    }
}
// check: ABORTED
// check: 3

//! new-transaction
//! sender: blessed
script{
    use 0x1::DualAttestation::{Self};
    fun main(not_blessed: &signer) {
        DualAttestation::set_microlibra_limit(not_blessed, 1001);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: blessed
script{
    use 0x1::DualAttestation;
    fun main(account: &signer) {
        DualAttestation::publish_credential(
            account, account,
            x"", x"", x""
        );
    }
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: blessed
script{
    use 0x1::DualAttestation;
    fun main(account: &signer) {
        DualAttestation::publish_credential(
            account, account,
            x"", x"", x""
        );
    }
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: libraroot
//! type-args: 0x1::Coin1::Coin1
//! args: 0, {{bob}}, {{bob::auth_key}}, b"bob", b"boburl", x"7013b6ed7dde3cfb1251db1b04ae9cd7853470284085693590a75def645a926d", true
stdlib_script::create_parent_vasp_account
// check: EXECUTED

//! new-transaction
//! sender: bob
script{
    use 0x1::DualAttestation;
    fun main(account: &signer) {
        DualAttestation::publish_credential(
            account, account,
            x"", x"", x""
        );
    }
}
// check: "Keep(ABORTED { code: 3,"

//! new-transaction
//! sender: blessed
script{
    use 0x1::DualAttestation;
    fun main(account: &signer) {
        DualAttestation::rotate_base_url(account, x"");
    }
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: blessed
script{
    use 0x1::DualAttestation;
    fun main(account: &signer) {
        DualAttestation::rotate_compliance_public_key(account, x"");
    }
}
// check: "Keep(ABORTED { code: 2,"

//! new-transaction
//! sender: bob
script{
    use 0x1::DualAttestation;
    fun main(account: &signer) {
        DualAttestation::rotate_compliance_public_key(account, x"");
    }
}
// check: "Keep(ABORTED { code: 5,"

//! new-transaction
//! sender: bob
script{
    use 0x1::DualAttestation;
    fun main(account: &signer) {
        DualAttestation::initialize(account);
    }
}
// check: "Keep(ABORTED { code: 0,"

//! new-transaction
//! sender: bob
script{
    use 0x1::DualAttestation;
    fun main(account: &signer) {
        DualAttestation::initialize(account);
    }
}
// check: "Keep(ABORTED { code: 0,"

//! new-transaction
//! sender: association
script {
use 0x1::Libra;
use 0x1::LibraConfig::CreateOnChainConfig;
use 0x1::Roles;
use 0x1::LibraTimestamp;
fun main(account: &signer) {
    LibraTimestamp::reset_time_has_started_for_test();
    let r = Roles::extract_privilege_to_capability<CreateOnChainConfig>(account);
    Libra::initialize(account, &r);
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: CANNOT_WRITE_EXISTING_RESOURCE

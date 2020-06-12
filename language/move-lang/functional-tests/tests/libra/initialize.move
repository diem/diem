//! new-transaction
//! sender: config
script {
use 0x1::Libra;
use 0x1::LibraConfig::CreateOnChainConfig;
use 0x1::Roles;
fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<CreateOnChainConfig>(account);
    Libra::initialize(account, &r);
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: CANNOT_WRITE_EXISTING_RESOURCE

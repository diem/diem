//! new-transaction
script{
use 0x1::LibraVersion;
use 0x1::LibraConfig::CreateOnChainConfig;
use 0x1::Roles;
fun main(account: &signer) {
    let r = Roles::extract_privilege_to_capability<CreateOnChainConfig>(account);
    LibraVersion::initialize(account, &r);
    Roles::restore_capability_to_privilege(account, r);
}
}
// check: ABORTED
// check: 1

//! new-transaction
script{
use 0x1::LibraVersion;
fun main(account: &signer) {
    LibraVersion::set(account, 0);
}
}
// check: ABORTED
// check: 25

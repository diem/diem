//! new-transaction
script{
use 0x1::DiemVersion;
fun main(account: &signer) {
    DiemVersion::initialize(account);
}
}

//! new-transaction
script{
use 0x1::DiemVersion;
fun main(account: &signer) {
    DiemVersion::set(account, 0);
}
}

//! new-transaction
//! sender: diemroot
script{
use 0x1::DiemVersion;
fun main(account: &signer) {
    DiemVersion::set(account, 0);
}
}

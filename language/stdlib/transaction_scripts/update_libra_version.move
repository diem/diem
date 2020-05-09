script {
use 0x0::LibraVersion;

fun main(account: &signer, major: u64) {
    LibraVersion::set(major, account)
}
}

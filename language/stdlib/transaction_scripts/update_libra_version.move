script {
use 0x1::LibraVersion;

fun update_libra_version(account: &signer, major: u64) {
    LibraVersion::set(account, major)
}
}

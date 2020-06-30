script {
use 0x1::LibraVersion;

/// Update Libra version.
fun update_libra_version(account: &signer, major: u64) {
    LibraVersion::set(account, major)
}
}

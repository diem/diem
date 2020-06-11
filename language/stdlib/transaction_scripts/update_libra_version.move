script {
use 0x1::LibraVersion;

fun main(account: &signer, major: u64) {
    LibraVersion::set(account, major)
}
}

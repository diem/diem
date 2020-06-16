script {
use 0x1::LibraVMConfig;

fun modify_publishing_option(account: &signer, args: vector<u8>) {
    LibraVMConfig::set_publishing_option(account, args)
}
}

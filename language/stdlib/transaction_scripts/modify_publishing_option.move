script {
use 0x1::LibraVMConfig;

fun main(account: &signer, args: vector<u8>) {
    LibraVMConfig::set_publishing_option(account, args)
}
}

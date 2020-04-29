script {
use 0x0::LibraVMConfig;

fun main(args: vector<u8>) {
    LibraVMConfig::set_publishing_option(args)
}
}

script {
use 0x1::LibraVMConfig;

/// Modify publishing options. Takes the LCS bytes of a `VMPublishingOption` object as input.
fun modify_publishing_option(account: &signer, args: vector<u8>) {
    LibraVMConfig::set_publishing_option(account, args)
}
}

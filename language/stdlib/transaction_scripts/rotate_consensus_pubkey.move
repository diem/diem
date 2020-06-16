script {
use 0x1::LibraSystem;
use 0x1::Signer;
use 0x1::ValidatorConfig;

fun rotate_consensus_pubkey(account: &signer, new_key: vector<u8>) {
    ValidatorConfig::set_consensus_pubkey(account, Signer::address_of(account), new_key);
    LibraSystem::update_and_reconfigure(account);
}
}

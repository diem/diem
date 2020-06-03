script {
use 0x0::LibraSystem;
use 0x0::Signer;
use 0x0::ValidatorConfig;

fun main(account: &signer, new_key: vector<u8>) {
    ValidatorConfig::set_consensus_pubkey(account, Signer::address_of(account), new_key);
    LibraSystem::update_and_reconfigure(account);
}
}

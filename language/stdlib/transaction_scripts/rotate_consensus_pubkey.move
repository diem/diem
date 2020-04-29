script {
use 0x0::ValidatorConfig;
use 0x0::LibraSystem;

fun main (new_key: vector<u8>) {
    ValidatorConfig::rotate_consensus_pubkey_of_sender(new_key);
    LibraSystem::update_and_reconfigure();
}
}

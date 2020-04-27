fun main (new_key: vector<u8>) {
    0x0::ValidatorConfig::rotate_consensus_pubkey_of_sender(new_key);
    0x0::LibraSystem::update_and_reconfigure();
}

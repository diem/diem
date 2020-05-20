script {
use 0x0::LibraSystem;
use 0x0::Transaction;
use 0x0::ValidatorConfig;

fun main (new_key: vector<u8>) {
    ValidatorConfig::set_consensus_pubkey(Transaction::sender(), new_key);
    LibraSystem::update_and_reconfigure();
}
}

//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
script {
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::get_validator_config({{vivian}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::LibraAccount;
    use 0x0::LibraSystem;
    fun main(account: &signer) {
        let num_validators = LibraSystem::validator_set_size();
        0x0::Transaction::assert(num_validators == 1, 98);
        let index = 0;
        while (index < num_validators) {
            let addr = LibraSystem::get_ith_validator_address(index);
            LibraAccount::decertify<LibraAccount::ValidatorRole>(account, addr);
            index = index + 1;
        }
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::LibraSystem;
    fun main(account: &signer) {
        LibraSystem::update_and_reconfigure(account);
        let num_validators = LibraSystem::validator_set_size();
        0x0::Transaction::assert(num_validators == 0, 98);
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::get_validator_config({{vivian}});
    }
}
// check: ABORTED
// check: 33

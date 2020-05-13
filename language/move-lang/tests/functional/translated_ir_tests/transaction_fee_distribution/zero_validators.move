//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
script {
    use 0x0::LibraSystem;
    fun main() {
        let num_validators = LibraSystem::validator_set_size();
        let index = 0;
        while (index < num_validators) {
            let addr = LibraSystem::get_ith_validator_address(index);
            LibraSystem::remove_validator(addr);
            index = index + 1;
        }
    }
}

//! block-prologue
//! proposer-address: 0x0
//! block-time: 3
// check: ABORTED
// check: 0

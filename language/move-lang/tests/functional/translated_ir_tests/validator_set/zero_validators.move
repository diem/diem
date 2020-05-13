//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: association
script {
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::get_validator_info({{vivian}});
    }
}
// check: EXECUTED

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

//! new-transaction
//! sender: association
script {
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::update_and_reconfigure()
    }
}

//! new-transaction
//! sender: association
script {
    use 0x0::LibraSystem;
    fun main() {
        LibraSystem::get_validator_info({{vivian}});
    }
}
// check: ABORTED
// check: 19

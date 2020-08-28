//! account: vivian, 1000000, 0, validator

//! block-prologue
//! proposer: vivian
//! block-time: 3

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraSystem;
    fun main() {
        LibraSystem::get_validator_config({{vivian}});
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraSystem;
    fun main(account: &signer) {
        let num_validators = LibraSystem::validator_set_size();
        assert(num_validators == 1, 98);
        let index = 0;
        while (index < num_validators) {
            let addr = LibraSystem::get_ith_validator_address(index);
            LibraSystem::remove_validator(account, addr);
            index = index + 1;
        };
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: libraroot
script {
    use 0x1::LibraSystem;
    fun main() {
        LibraSystem::get_validator_config({{vivian}});
    }
}
// check: "Keep(ABORTED { code: 775,"

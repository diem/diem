module 0x42::TestModule {

    struct R has key { value: u64 }

    fun store(s: &signer, value: u64) {
       move_to<R>(s, R{value})
    }
}

spec 0x42::TestModule {
    spec store(s: &signer) {
    }

    spec store_undefined(s: &signer, value: u64) {
    }
}

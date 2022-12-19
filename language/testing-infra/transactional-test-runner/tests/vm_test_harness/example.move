//# init --addresses A=0x42

//# publish
module A::N {
}

//# run

script {
    fun main() {}
}

//# view
//#     --address 0x1
//#     --resource 0x42::N::R<u64>

//# publish
module A::N {
    struct R<V: store> has key {
        v: V
    }

    public fun give(s: &signer) {
        move_to(s, R { v: 0 })
    }

    public fun take(s: &signer): u64 acquires R {
        let R { v } = move_from(Std::Signer::address_of(s));
        v
    }

    public(script) fun ex(_s: signer, _u: u64) {
        abort 0
    }
}

//# run --signers 0x1 --args 0 -- 0x42::N::ex

//# run --signers 0x1

script {
    fun main(s: signer) {
        A::N::give(&s)
    }
}

//# view
//#     --address 0x1
//#     --resource 0x42::N::R<u64>

//# run --signers 0x1 --syntax=mvir

import 0x42.N;
main(s: signer) {
    _ = N.take(&s);
    return;
}

//# view
//#     --address 0x1
//#     --resource 0x42::N::R<u64>

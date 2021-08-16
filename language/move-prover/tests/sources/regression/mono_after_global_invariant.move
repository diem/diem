address 0x2 {
module Base {
    struct B has key {}

    public fun BASE_ADDR(): address {
        @0x2
    }

    public fun put_b(s: &signer) {
        move_to(s, B {});
        // the global invariants in 0x2::Test is instrumented here
    }

    spec module {
        fun has_b(): bool {
            exists<B>(BASE_ADDR())
        }
    }
}

module Test {
    use 0x2::Base;

    struct R<T: store> has key {
         f: T,
    }

    public fun put_r<T: store>(s: &signer, v: T) {
        Base::put_b(s);
        move_to(s, R { f: v });
    }

    spec module {
        fun has_r<T>(): bool {
            exists<R<T>>(Base::BASE_ADDR())
        }
    }

    spec module {
        invariant<T> update
            Base::has_b() ==> (has_r<T>() ==> old(has_r<T>()));

        // The above invariant should not verify, here is a counterexample:
        //
        // suppose we starts with an empty state,
        // put_r(@0x2, false) will violate the invariant, because
        // - Base::has_b() is true,
        // - has_r<bool>() is true, but
        // - old(has_r<bool>()) is false
    }
}
}

module 0x42::TestAxioms {

    spec module {
        pragma verify = true;
    }

    spec module {
        fun spec_incr(x: num): num;
        axiom forall x: num: spec_incr(x) == x + 1;
    }

    fun incr(x: u64): u64 {
        x + 1
    }
    spec incr {
        ensures result == TRACE(spec_incr(x));
    }

    // TODO: reactivate this test when generic axiom instantiation is done
    /*
    // Axiom over generic function, using type quantification which is expected to be eliminated.
    spec module {
        fun spec_id<T>(x: T): T;
        axiom forall t: type, x: t: spec_id(x) == x;
    }

    fun id_T<T>(x: T): T {
        x
    }
    spec id_T {
        ensures result == spec_id(x);
    }

    fun id_u64<T>(x: u64): u64 {
        x
    }
    spec id_u64 {
        ensures result == spec_id(x);
    }
    */
}

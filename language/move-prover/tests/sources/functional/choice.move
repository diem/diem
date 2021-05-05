// also_include_for: cvc4
module 0x42::TestSome {
    use 0x1::Signer;
    use 0x1::Vector;
    // Basic tests
    // ===========

    fun simple(): u64 {
        4
    }
    spec fun simple {
        ensures result <= (choose x: u64 where x >= 4);
    }

    fun simple_incorrect(b: bool): u64 {
        if (b) 4 else 5
    }
    spec fun simple_incorrect {
        // This fails because the interpretation is not influenced by an assertion.
        // The choice freely selects a value and not a 'fitting' one.
        ensures result == TRACE(choose x: u64 where x >= 4 && x <= 5);
    }

    // Testing choices in spec functions
    // =================================

    spec define spec_fun_choice(x: u64): u64 {
        choose y: u64 where y >= x
    }
    fun with_spec_fun_choice(x: u64): u64 {
        x + 42
    }
    spec fun with_spec_fun_choice {
        ensures result <= TRACE(spec_fun_choice(x + 42));
    }

    // Testing choices using memory
    // ============================

    struct R has key {
        x: u64
    }

    fun populate_R(s1: &signer, s2: &signer) {
        move_to<R>(s1, R{x: 1});
        move_to<R>(s2, R{x: 2});
    }
    spec fun populate_R {
        let a1 = Signer::spec_address_of(s1);
        let a2 = Signer::spec_address_of(s2);
        /// The requires guarantees that there is no other address which can satisfy the choice below.
        requires forall a: address: !exists<R>(a);
        let choice = choose a: address where exists<R>(a) && global<R>(a).x == 2;
        ensures choice == Signer::spec_address_of(s2);
    }

    // Testing min choice
    // ==================

    fun test_min(): vector<u64> {
        let v = Vector::empty<u64>();
        let v_ref = &mut v;
        Vector::push_back(v_ref, 1);
        Vector::push_back(v_ref, 2);
        Vector::push_back(v_ref, 3);
        Vector::push_back(v_ref, 2);
        v
    }
    spec fun test_min {
        ensures (choose min i in 0..len(result) where result[i] == 2) == 1;
    }

    fun test_not_using_min_incorrect(): vector<u64> {
        let v = Vector::empty<u64>();
        let v_ref = &mut v;
        Vector::push_back(v_ref, 1);
        Vector::push_back(v_ref, 2);
        Vector::push_back(v_ref, 3);
        Vector::push_back(v_ref, 2);
        Vector::push_back(v_ref, 2);
        v
    }
    spec fun test_not_using_min_incorrect {
        // This fails because we do not necessary select the smallest i
        ensures TRACE(choose i in 0..len(result) where result[i] == 2) == 1;
    }

}

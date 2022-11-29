// separate_baseline: cvc4
// TODO(cvc4): this test requires a separate baseline because cvc4 produces false positives for some of choices
module 0x42::TestSome {
    use Std::Signer;
    use Std::Vector;

    // Basic tests
    // ===========

    fun simple(): u64 { 4 }
    spec simple {
        ensures result <= (choose x: u64 where x >= 4);
    }

    fun simple_incorrect(b: bool): u64 {
        if (b) 4 else 5
    }
    spec simple_incorrect {
        // This fails because the interpretation is not influenced by an assertion.
        // The choice freely selects a value and not a 'fitting' one.
        ensures result == TRACE(choose x: u64 where x >= 4 && x <= 5);
    }

    // Testing choices in spec funs
    // =============================

    spec fun spec_fun_choice(x: u64): u64 {
        choose y: u64 where y >= x
    }
    fun with_spec_fun_choice(x: u64): u64 {
        x + 42
    }
    spec with_spec_fun_choice {
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
    spec populate_R {
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
    spec test_min {
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
    spec test_not_using_min_incorrect {
        // This fails because we do not necessary select the smallest i
        ensures TRACE(choose i in 0..len(result) where result[i] == 2) == 1;
    }

    // Testing choice duplication
    // ==========================

    // This is only a compilation test. It fails verification.

    fun test_choice_dup_expected_fail(x: u64): u64 {
        x + 1
    }
    spec test_choice_dup_expected_fail {
        pragma opaque; // making this opaque lets the choice be injected at each call
        ensures result == TRACE(choose y: u64 where y > x);
    }

    fun test_choice_use1(a: u64): u64 {
        test_choice_dup_expected_fail(a)
    }

    fun test_choice_use2(_a: vector<u64>, b: u64): u64 {
        // with incorrect use of parameters, this would use $t0 as a parameter to the choice
        // function, which leads to a type error in boogie.
        test_choice_dup_expected_fail(b)
    }
}

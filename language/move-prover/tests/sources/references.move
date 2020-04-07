// dep: tests/sources/stdlib/modules/vector.move
module TestReferences {
    use 0x0::Vector;

    // ------------------------
    // References as parameters
    // ------------------------

    struct T {
        a: u64
    }

    fun ref_param(r: &T): u64 {
        r.a
    }
    spec fun ref_param {
        ensures result == r.a;
    }

    fun ref_param_vec(r: &vector<T>): u64 {
        Vector::length(r)
    }
    spec fun ref_param_vec {
        ensures result == len(r);
    }

    fun ref_return(r: &vector<T>, i: u64): &T {
        Vector::borrow(r, i)
    }
    spec fun ref_return {
        ensures result == r[i];
    }


    // -----------------------------
    // References as local variables
    // -----------------------------

    fun mut_b(b: &mut u64) {
        *b = 10;
    }

    fun mut_ref() {
        let b: u64 = 20;
        let b_ref: &mut u64 = &mut b;
        mut_b(b_ref);
        b = *b_ref;
        if (b != 10) abort 1;
    }
    spec fun mut_ref {
        aborts_if false;
    }

    fun mut_ref_incorrect() {
        let b: u64 = 20;
        let b_ref: &mut u64 = &mut b;
        mut_b(b_ref);
        b = *b_ref;
        if (b != 10) abort 1;
    }
    spec fun mut_ref_incorrect {
        aborts_if true;
    }


    // ---------------------------
    // References as return values
    // ---------------------------

    resource struct WithdrawalCapability {
        account_address: address,
    }

    fun withdrawal_capability_address(cap: &WithdrawalCapability): &address {
        &cap.account_address
    }
    spec fun withdrawal_capability_address {
        ensures result == cap.account_address;
    }
}

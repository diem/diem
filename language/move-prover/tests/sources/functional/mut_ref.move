module 0x1::TestMutRef {
    use Std::Vector;

    struct T has copy, drop { value: u64 }
    struct R has copy, drop { value: u64 }
    struct N has copy, drop { value: u64, t: T }

    spec T { invariant value > 0; }

    // Identity
    // ========

    fun identity(x: &mut u64): &mut u64 { x }

    fun call_identity(): u64 {
        let x = 1u64;
        let r = identity(&mut x);
        *r = 2;
        x
    }
    spec call_identity {
        ensures result == 2;
    }

    // Return reference with different root
    // ====================================

    fun return_ref_different_root(b: bool, x: &mut T, y: &mut R): &mut u64 {
        if (b) &mut x.value else &mut y.value
    }

    fun call_return_ref_different_root(b: bool): (T, R) {
        let x = T{value: 1};
        let y = R{value: 10};
        let r = return_ref_different_root(b, &mut x, &mut y);
        *r = 5;
        (x, y)
    }
    spec call_return_ref_different_root {
        ensures b ==> result_1 == T{value: 5} && result_2 == R{value: 10};
        ensures !b ==> result_1 == T{value: 1} && result_2 == R{value: 5};
    }

    // Return reference with different path
    // ====================================

    fun return_ref_different_path(b: bool, x: &mut N): &mut u64 {
        if (b) &mut x.value else &mut x.t.value
    }

    fun call_return_ref_different_path(b: bool): N {
        let x = N{value: 1, t: T{value: 2}};
        let r = return_ref_different_path(b, &mut x);
        *r = 5;
        x
    }
    spec call_return_ref_different_path {
        ensures b ==> result == N{value: 5, t: T{value: 2}};
        ensures !b ==> result == N{value: 1, t: T{value: 5}};
    }

    // Return reference with different path, vector
    // ============================================

    struct V has copy, drop { is: vector<u64>, ts: vector<T> }

    // Different path into one vector

    fun return_ref_different_path_vec(b: bool, x: &mut V): &mut u64 {
        if (b) Vector::borrow_mut(&mut x.is, 1)  else Vector::borrow_mut(&mut x.is, 0)
    }

    fun call_return_ref_different_path_vec(b: bool): V {
        let is = Vector::empty();
        let ts = Vector::empty();
        Vector::push_back(&mut is, 1);
        Vector::push_back(&mut is, 2);
        let x = V{is, ts};
        let r = return_ref_different_path_vec(b, &mut x);
        *r = 5;
        x
    }
    spec call_return_ref_different_path_vec {
        ensures b ==> result == V{is: concat(vec(1u64), vec(5u64)), ts: vec()};
        ensures !b ==> result == V{is: concat(vec(5u64), vec(2u64)), ts: vec()};
    }

    // Different path into a vector or a vector of structs subfield

    fun return_ref_different_path_vec2(b: bool, x: &mut V): &mut u64 {
        if (b) Vector::borrow_mut(&mut x.is, 1) else &mut (Vector::borrow_mut(&mut x.ts, 0)).value
    }

    fun call_return_ref_different_path_vec2(b: bool): V {
        let is = Vector::empty();
        let ts = Vector::empty();
        Vector::push_back(&mut is, 1);
        Vector::push_back(&mut is, 2);
        Vector::push_back(&mut ts, T{value: 3});
        Vector::push_back(&mut ts, T{value: 4});
        let x = V{is, ts};
        let r = return_ref_different_path_vec2(b, &mut x);
        *r = 5;
        x
    }
    spec call_return_ref_different_path_vec2 {
        ensures b ==> result == V{is: concat(vec(1u64), vec(5u64)), ts: concat(vec(T{value: 3}), vec(T{value: 4}))};
        ensures !b ==> result == V{is: concat(vec(1u64), vec(2u64)), ts: concat(vec(T{value: 5}), vec(T{value: 4}))};
    }

    // Some as above but with invariant violation.

    fun call_return_ref_different_path_vec2_incorrect(b: bool): V {
        let is = Vector::empty();
        let ts = Vector::empty();
        Vector::push_back(&mut is, 1);
        Vector::push_back(&mut is, 2);
        Vector::push_back(&mut ts, T{value: 3});
        Vector::push_back(&mut ts, T{value: 4});
        let x = V{is, ts};
        let r = return_ref_different_path_vec2(b, &mut x);
        *r = 0;
        x
    }
    spec call_return_ref_different_path_vec2_incorrect {
        ensures b ==> result == V{is: concat(vec(1u64), vec(0u64)), ts: concat(vec(T{value: 3}), vec(T{value: 4}))};
        ensures !b ==> result == V{is: concat(vec(1u64), vec(2u64)), ts: concat(vec(T{value: 0}), vec(T{value: 4}))};
    }

}

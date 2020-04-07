// dep: tests/sources/stdlib/modules/vector.move

// This file consists of a series of test cases which are client functions
// using the standard vector module.

module TestVector {
    use 0x0::Vector;


    // -----------------------------
    // Testing with concrete vectors
    // -----------------------------

    fun test_vector_equal(v: vector<u64>, w: &mut vector<u64>) {
    }
    spec fun test_vector_equal {
        aborts_if false;
        ensures v == v;
        ensures old(v) == old(v);
        ensures v == v[0..len(v)];
        ensures old(v) == old(v[0..len(v)]);
        ensures w == w;
        ensures old(w) == old(w);
        ensures w == w[0..len(w)];
        ensures old(w) == old(w[0..len(w)]);
    }

    // succeeds. [] == [].
    fun test_empty() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        (ev1, ev2)
    }
    spec fun test_empty {
        ensures result_1 == result_2;
        ensures len(result_1) == 0;
        ensures len(result_2) == 0;
    }

    // succeeds. [1] == [1]
    fun test_push() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::push_back(&mut ev2, 1);
        (ev1, ev2)
    }
    spec fun test_push {
        ensures result_1 == result_2;
        ensures len(result_1) == 1;
        ensures len(result_2) == 1;
    }

    //succeeds. [] == [].
    fun test_push_pop() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::pop_back(&mut ev1);
        (ev1, ev2)
    }
    spec fun test_push_pop {
        ensures result_1 == result_2;
    }

    //succeeds. [1,2] != [1].
    fun test_neq1() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::push_back(&mut ev1, 2);
        Vector::push_back(&mut ev2, 1);
        (ev1, ev2)
    }
    spec fun test_neq1 {
        ensures result_1 != result_2;
    }

    // succeeds. [1] == [0]
    fun test_neq2() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::push_back(&mut ev2, 0);
        (ev1, ev2)
    }
    spec fun test_neq2 {
        ensures result_1 != result_2;
    }

    // succeeds. reverse([]) == [].
    fun test_reverse1() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::reverse(&mut ev1);
        (ev1, ev2)
    }
    spec fun test_reverse1 {
        ensures result_1 == result_2;
    }

    // succeeds. reverse([1,2]) == [2,1].
    fun test_reverse2() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::push_back(&mut ev1, 2);
        Vector::push_back(&mut ev2, 2);
        Vector::push_back(&mut ev2, 1);
        Vector::reverse(&mut ev1);
        (ev1, ev2)
    }
    spec fun test_reverse2 {
        ensures result_1 == result_2;
    }

    // succeeds. reverse([1,2]) != [1,2].
    fun test_reverse3() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::push_back(&mut ev1, 2);
        Vector::push_back(&mut ev2, 1);
        Vector::push_back(&mut ev2, 2);
        Vector::reverse(&mut ev1);
        (ev1, ev2)
    }
    spec fun test_reverse3 {
        ensures result_1 != result_2;
    }

    // succeeds. swap([1,2],0,1) == [2,1].
    fun test_swap() : (vector<u64>, vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::push_back(&mut ev1, 2);
        Vector::push_back(&mut ev2, 2);
        Vector::push_back(&mut ev2, 1);
        Vector::swap(&mut ev1, 0, 0);
        Vector::swap(&mut ev1, 0, 1);
        (ev1, ev2)
    }
    spec fun test_swap {
        ensures result_1 == result_2;
    }

    // succeeds. Always aborts because the first index argument of `swap` is out-of-bounds.
    fun test_swap_abort1()
    {
        let ev1 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::swap(&mut ev1, 1, 0);
    }
    spec fun test_swap_abort1 {
        aborts_if true;
    }

    // succeeds. Always aborts because the second index argument of `swap` is out-of-bounds.
    fun test_swap_abort2()
    {
        let ev1 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::swap(&mut ev1, 0, 1);
    }
    spec fun test_swap_abort2 {
        aborts_if true;
    }

    // succeeds. len([1]) = len([]) + 1.
    fun test_length1() : (u64, u64)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        (Vector::length(&ev1), Vector::length(&ev2))
    }
    spec fun test_length1 {
        ensures result_1 == result_2 + 1;
    }

    fun vector_of_proper_positives(): vector<u64> {
        let v = Vector::empty();
        Vector::push_back(&mut v, 1);
        Vector::push_back(&mut v, 2);
        Vector::push_back(&mut v, 3);
        v
    }
    spec fun vector_of_proper_positives {
      ensures all(result, |n| n > 0);
      ensures all(0..len(result), (|i| all(0..len(result), (|j| result[i] == result[j] ==> i == j))));
    }

    // succeeds. 7 == 7.
    fun test_borrow1() : u64
    {
        let v = Vector::empty<u64>();
        Vector::push_back(&mut v, 7);
        *Vector::borrow(&v, 0)
    }
    spec fun test_borrow1 {
        ensures result == 7;
    }

    // succeeds. 7 != 7.
    fun test_borrow2() : u64
    {
        let v = Vector::empty<u64>();
        Vector::push_back(&mut v, 0);
        *Vector::borrow(&v, 0)
    }
    spec fun test_borrow2 {
        ensures result != 7;
    }

    // succeeds. Always aborts due to the out-of-bounds index used.
    fun test_borrow3() : u64
    {
        let v = Vector::empty<u64>();
        Vector::push_back(&mut v, 7);
        *Vector::borrow(&v, 1)
    }
    spec fun test_borrow3 {
        aborts_if true;
    }

    fun test_slice() : (vector<u64>,vector<u64>)
    {
        let ev1 = Vector::empty<u64>();
        let ev2 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::push_back(&mut ev1, 2);
        Vector::push_back(&mut ev2, 0);
        Vector::push_back(&mut ev2, 1);
        Vector::push_back(&mut ev2, 2);
        Vector::push_back(&mut ev2, 3);
        Vector::push_back(&mut ev2, 1);
        Vector::push_back(&mut ev2, 2);
        (ev1, ev2)
    }
    spec fun test_slice {
        ensures result_1 == result_2[1..3];
        ensures result_1 != result_2[0..2];
        ensures result_1 == result_2[4..6];
        ensures result_1[0..2] == result_2[4..6];
        ensures result_1[1..2] == result_2[2..3];
        ensures result_2[1..3] == result_2[4..6];
    }

    fun test_contains() : (vector<u64>, bool, bool)
    {
        let b1: bool;
        let b2: bool;
        let ev1 = Vector::empty<u64>();
        Vector::push_back(&mut ev1, 1);
        Vector::push_back(&mut ev1, 2);
        Vector::push_back(&mut ev1, 3);
        Vector::push_back(&mut ev1, 5);
        b1 = Vector::contains(&ev1, &3);
        b2 = Vector::contains(&ev1, &4);
        (ev1, b1, b2)
    }
    spec fun test_contains {
        aborts_if false;
        ensures result_2 == true;
        ensures result_3 == false;
        ensures len(result_1) == 4;
        ensures result_1[0] == 1;
        ensures result_1[1] == 2;
        ensures result_1[2] == 3;
        ensures result_1[3] == 5;
        ensures any(result_1,|x| x==1);
        ensures any(result_1,|x| x==2);
        ensures any(result_1,|x| x==3);
        ensures !any(result_1,|x| x==4);
        ensures any(result_1,|x| x==5);
    }


    // ---------------------------
    // Testing functions with args
    // ---------------------------

    fun test_length2(v: vector<u64>) : (u64, u64)
    {
        let x: u64;
        let y: u64;
        x = Vector::length(&v);
        Vector::push_back(&mut v, 1);
        Vector::push_back(&mut v, 2);
        Vector::push_back(&mut v, 3);
        y = Vector::length(&v);
        (x, y)
    }
    spec fun test_length2 {
        ensures result_1 + 3 == result_2;
    }

    fun test_length3(v: vector<u64>) : (u64, u64)
    {
        let l = Vector::length(&v);
        Vector::push_back(&mut v, 1);
        (l, Vector::length(&v))
    }
    spec fun test_length3 {
        ensures len(old(v)) == result_1;
        ensures result_1 + 1 == result_2;
        ensures v == old(v);            // TODO: issue of using mutated params in spec
        ensures len(v) != result_2;     // TODO: issue of using mutated params in spec
    }

    fun test_length4(v: &mut vector<u64>) : (u64, u64)
    {
        let l = Vector::length(v);
        Vector::push_back(v, 1);
        (l, Vector::length(v))
    }
    spec fun test_length4 {
        ensures len(old(v)) == result_1;
        ensures result_1 + 1 == result_2;
        ensures v != old(v);
        ensures len(v) == result_2;
    }

    // succeeds. v == v.
    fun test_id1(v: vector<u64>) : vector<u64>
    {
        v
    }
    spec fun test_id1 {
        ensures result == v;
    }

    // succeeds. reverse(reverse(v)) == v.
    fun test_id2(v: vector<u64>) : vector<u64>
    {
        Vector::reverse(&mut v);
        Vector::reverse(&mut v);
        v
    }
    spec fun test_id2 {
        ensures result == v;
    }

    // succeeds. reverse(some_obscure_reverse_routine(v)) == v.
    fun test_id3(v: vector<u64>) : vector<u64>
    {
        let l: u64 = Vector::length(&v);
        if(l <= 1) {
        }
        else {
            if(l <= 3) {
                Vector::swap(&mut v, 0, l-1);
            }
            else {
                Vector::reverse(&mut v);
            }
        };
        Vector::reverse(&mut v);
        v
    }
    spec fun test_id3 {
        ensures result == v;
    }

    // succeeds. If the input vector is empty, destroy it, and return a new empty vector.
    fun test_destroy_empty1(v: vector<u64>) : vector<u64>
    {
        if(Vector::is_empty(&v)) {
            Vector::destroy_empty(v);
            Vector::empty<u64>()
        }
        else {
            v
        }
    }
    spec fun test_destroy_empty1 {
        ensures result == v;
    }

    // succeeds. If the input vector is empty, destroy it, and return a new empty vector.
    fun test_destroy_empty2(v: vector<u64>)
    {
        if(Vector::is_empty(&v)) {
            Vector::swap(&mut v, 0, 0);
        }
        else {
            Vector::destroy_empty(v);
        }
    }
    spec fun test_destroy_empty2 {
        aborts_if true;
    }

    fun test_borrow_mut(v: &mut vector<u64>) : u64
    {
        let x = *Vector::borrow(v, 0);
        *Vector::borrow_mut(v, 0) = 7;
        x
    }
    spec fun test_borrow_mut {
        aborts_if len(old(v)) == 0;
    }


    // --------------------------------------------
    // Custom Option type using vector as container
    // --------------------------------------------

    struct T<E> {
        v: vector<E>
    }

    fun none<E>(): T<E> {
        T<E> {v: Vector::empty<E>()}
    }

    fun some<E>(e: E): T<E> {
        let v = Vector::empty<E>();
        Vector::push_back(&mut v, e);
        T<E> {v: v}
    }

    fun unwrap_or<E: copyable>(x: T<E>, e: E): E {
        let T<E> {v : v} = x;
        if (Vector::is_empty<E>(&v))
            e
        else
            Vector::pop_back<E>(&mut v)
    }

    fun option_type(): (u64, u64) {
        let n = none<u64>();
        let s = some<u64>(42);
        (unwrap_or<u64>(n, 0), unwrap_or<u64>(s, 0))
    }
    spec fun option_type {
        ensures result_1 == 0;
        ensures result_2 == 42;
    }
}

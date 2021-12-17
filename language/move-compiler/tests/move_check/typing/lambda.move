module 0x8675309::M {
    use 0x1::XVector;

    public macro foreach<T>(v: &vector<T>, action: |&T|) { // expected to be not implemented
        let i = 0;
        while (i < XVector::length(v)) {
            action(XVector::borrow(v, i));
            i = i + 1;
        }
    }

    public macro reduce<R, T>(v: vector<T>, accu: R, reducer: |T, R|R): R {
        while (!XVector::is_empty(&v)) {
            accu = reducer(XVector::pop_back(&mut v), accu);
        };
        accu
    }


    public fun correct_foreach() {
        let v = vector[1, 2, 3];
        let sum = 0;
        foreach!(&v, |e| sum = sum + *e) // expected to be not implemented
    }

    public fun correct_reduce(): u64 {
        let v = vector[1, 2, 3];
        reduce!(v, 0, |t, r| t + r)
    }

    public fun corrected_nested() {
        let v = vector[vector[1,2], vector[3]];
        let sum = 0;
        foreach!(&v, |e| sum = sum + reduce!(*e, 0, |t, r| t + r));
    }

    public macro wrong_local_call_arg_count<T>(v: &vector<T>, action: |&T|) {
        let i = 0;
        while (i < XVector::length(v)) {
            action(XVector::borrow(v, i), i); // expected to have wrong argument count
            i = i + 1;
        }
    }

    public macro wrong_local_call_arg_type<T>(v: &vector<T>, action: |&T|) {
        let i = 0;
        while (i < XVector::length(v)) {
            action(i); // expected to have wrong argument type
            i = i + 1;
        }
    }

    public macro wrong_local_call_result_type<T>(v: &vector<T>, action: |&T|) {
        let i = 0;
        while (i < XVector::length(v)) {
            i = i + action(XVector::borrow(v, i)); // expected to have wrong result type
        }
    }

    public fun wrong_local_call_no_fun(x: u64) {
        x(1)
    }

    public fun wrong_lambda_inferred_type() {
        let v = vector[1, 2, 3];
        let sum = 0;
        foreach!(&v, |e| sum = sum + e) // expected to cannot infer type
    }

    public fun wrong_lambda_result_type() {
        let v = vector[1, 2, 3];
        let sum = 0;
        foreach!(&v, |e| { sum = sum + *e; *e }) // expected to have wrong result type of lambda
    }

    public fun lambda_not_allowed() {
        let _x = |i| i + 1; // expected lambda not allowed
    }

    struct FieldFunNotAllowed {
        f: |u64|u64, // expected lambda not allowed
    }

    public fun fun_arg_lambda_not_allowed(x: |u64|) {} // expected lambda not allowed

    public macro macro_result_lambda_not_allowed(): |u64| {  // expected lambda not allowed
        abort (1)
    }
    public fun fun_result_lambda_not_allowed(): |u64| {  // expected lambda not allowed
        abort (1)
    }

}

module 0x1::XVector {
    public fun length<T>(v: &vector<T>): u64 { abort(1) }
    public fun is_empty<T>(v: &vector<T>): bool { abort(1) }
    public fun borrow<T>(v: &vector<T>, i: u64): &T { abort(1) }
    public fun pop_back<T>(v: &mut vector<T>): T { abort(1) }
}

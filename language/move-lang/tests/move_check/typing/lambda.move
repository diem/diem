module 0x8675309::M {
    use 0x1::XVector;

    public macro foreach<T>(v: &vector<T>, action: |&T|) { // expected to be not implemented
        let i = 0;
        while (i < XVector::length(v)) {
            action(XVector::borrow(v, i));
            i = i + 1;
        }
    }

    public fun correct() {
        let v = vector[1, 2, 3];
        let sum = 0;
        foreach!(&v, |e| sum = sum + *e) // expected to be not implemented
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
        foreach!(&v, |e| sum = sum + e) // expected
    }

    public fun wrong_lambda_result_type() {
        let v = vector[1, 2, 3];
        let sum = 0;
        foreach!(&v, |e| { sum = sum + *e; *e }) // expected
    }

    public fun lambda_not_allowed() {
        let _x = |i| i + 1; // expected
    }

    struct FieldFunNotAllowed {
        f: |u64|u64, // expected
    }

    public fun fun_arg_not_allowed(x: |u64|) {} // expected
}

module 0x1::XVector {
    public fun length<T>(v: &vector<T>): u64 { abort(1) }
    public fun borrow(v: &vector<T>, i: u64): &T { abort(1) }

}

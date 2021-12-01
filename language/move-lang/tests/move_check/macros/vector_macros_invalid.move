module A::Test {
    use Std::Vector;

    fun simple_name(v: &mut vector<u64>) {
        foreach_mut!(v, |x| *x = *x + 1)
    }

    fun numeric_address(v: &mut vector<u64>) {
        0x1::Vector::foreach_mut!(v, |x| *x = *x + 1)
    }

    fun undefined_macro(v: &mut vector<u64>) {
        Vector::foreachx!(v, |x| *x = *x + 1)
    }

    fun wrong_number_of_args(v: &mut vector<u64>) {
        Vector::foreach_mut!(v)
    }

    fun not_a_lambda(v: &mut vector<u64>) {
        foreach_mut!(v, v)
    }

    fun misplaced_lambda(v: &mut vector<u64>) {
        Vector::foreach_mut!(|x| *x = *x + 1, |x| *x = *x + 1)
    }

    fun lambda_wrong_number_of_args(v: &mut vector<u64>) {
        Vector::foreach_mut!(v, |x, y| *x = *y + 1)
    }

    fun resolves_but_wrong_vector(v: &mut vector<u64>) {
        Vector::foreach_mut!(v, |x| *x = *x + 1)
    }
}

// Mock for the vector module
module Std::Vector {
    public fun lengthx<T>(_v: &vector<T>): u64 { abort(1) }
    public fun borrow_mut<T>(_v: &mut vector<T>, _i: u64): &mut T { abort(1) }
}

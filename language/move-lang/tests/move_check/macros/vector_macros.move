module A::Test {
    use Std::Vector;

    fun increment_vec(v: &mut vector<u64>) {
        Vector::foreach_mut!(v, |x| *x = *x + 1)
    }

    fun increment_vec_nested(v: &mut vector<vector<u64>>) {
        Vector::foreach_mut!(v, |nested| Vector::foreach_mut!(nested, |x| *x = *x + 1))
    }
}

module A::Test2 {
    fun increment_vec(v: &mut vector<u64>) {
        Std::Vector::foreach_mut!(v, |x| *x = *x + 1)
    }
}

// Mock for the vector module
module Std::Vector {
    public fun length<T>(_v: &vector<T>): u64 { abort(1) }
    public fun borrow<T>(_v: &vector<T>, _i: u64): &T { abort(1) }
    public fun borrow_mut<T>(_v: &mut vector<T>, _i: u64): &mut T { abort(1) }
}

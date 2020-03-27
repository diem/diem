// dep: tests/sources/modules/vector.move
module TestReferences {
    use 0x0::Vector;

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
}

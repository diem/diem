module M {
    struct X { y: Y }
    struct Y { u: u64 }

    fun t1(): &u64 {
        let x = 0;
        &x
    }

    fun t2(): &u64 {
        let x = 0;
        let y = &x;
        copy y
    }

    fun t3(): &u64 {
        let s = X { y: Y { u: 0 } };
        let x = &s;
        let y = &x.y;
        let u = &y.u;
        move u
    }

    fun t4(): &u64 {
        let s = X { y: Y { u: 0 } };
        let x = &s;
        let y = &x.y;
        let u = &y.u;
        copy u
    }
}

// check: RET_UNSAFE_TO_DESTROY_ERROR
// check: RET_UNSAFE_TO_DESTROY_ERROR
// check: RET_UNSAFE_TO_DESTROY_ERROR
// check: RET_UNSAFE_TO_DESTROY_ERROR

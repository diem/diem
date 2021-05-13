module 0x2::A {
    use Std::Vector;

    struct X has copy, drop { value: u64 }
    struct Y has copy, drop { value: u64 }
    struct Z has copy, drop { value: u64, x: X }
    struct V has copy, drop { is: vector<u64>, ts: vector<X> }

    // Return reference with different root
    fun return_ref_different_root(cond: bool, x: &mut X, y: &mut Y): &mut u64 {
        if (cond) &mut x.value else &mut y.value
    }

    #[test]
    public fun return_ref_root(): (X, Y) {
        let x = X { value: 1 };
        let y = Y { value: 2 };
        let p = return_ref_different_root(true, &mut x, &mut y);
        *p = 5;
        let q = return_ref_different_root(false, &mut x, &mut y);
        *q = 6;
        (x, y)
    }

    // Return reference with different path
    fun return_ref_different_path(cond: bool, z: &mut Z): &mut u64 {
        if (cond) &mut z.value else &mut z.x.value
    }

    #[test]
    public fun return_ref_path(): (Z, Z) {
        let z1 = Z { value: 1, x: X { value: 2 } };
        let p = return_ref_different_path(true, &mut z1);
        *p = 5;
        let z2 = Z { value: 1, x: X { value: 2 } };
        let q = return_ref_different_path(false, &mut z2);
        *q = 6;
        (z1, z2)
    }

    // Return reference with different path in vec
    fun return_ref_different_path_vec(cond: bool, v: &mut V, i1: u64, i2: u64): &mut u64 {
        if (cond) {
            Vector::borrow_mut(&mut v.is, i1)
        } else {
            Vector::borrow_mut(&mut v.is, i2)
        }
    }

    // TODO there is a bug in spec_instrumenter that produces wrong goto labels:
    // #[test]
    public fun return_ref_path_vec_1(): V {
        let is = Vector::empty();
        let ts = Vector::empty();
        Vector::push_back(&mut is, 1);
        Vector::push_back(&mut is, 2);
        let v = V { is, ts };
        let p = return_ref_different_path_vec(true, &mut v, 0, 1);
        *p = 5;
        let q = return_ref_different_path_vec(false, &mut v, 0, 1);
        *q = 6;
        v
    }
}

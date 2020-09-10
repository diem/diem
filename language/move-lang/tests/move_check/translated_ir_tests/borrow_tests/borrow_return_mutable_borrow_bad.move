module M {
    struct X { f: Y }
    struct Y { g: u64, h: u64 }

    fun t1(ref_x: &mut X): (&mut Y, &u64)  {
        let ref_x_f = &mut ref_x.f;
        let ref_x_f_g = &ref_x_f.g;

        (ref_x_f, ref_x_f_g)
    }
}

// check: RET_BORROWED_MUTABLE_REFERENCE_ERROR

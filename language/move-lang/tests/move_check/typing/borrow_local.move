module M {
    struct S {}
    resource struct R {}

    fun t0(b: bool, u: u64, s: S, r: R): R {
        (&b : &bool);
        (&mut b : &mut bool);
        (&u : &u64);
        (&mut u : &mut u64);
        (&s : &S);
        (&mut s : &mut S);
        (&r: &R);
        (&mut r: &mut R);
        r
    }

    fun t1(): R {
        let b = true;
        let u = 0;
        let s = S {};
        let r = R {};

        (&b : &bool);
        (&mut b : &mut bool);
        (&u : &u64);
        (&mut u : &mut u64);
        (&s : &S);
        (&mut s : &mut S);
        (&r: &R);
        (&mut r: &mut R);
        r
    }

}

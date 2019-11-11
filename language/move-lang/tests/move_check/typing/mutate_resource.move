module M {
    resource struct R {}

    t0(r: &mut R) {
        *r = R {};
    }

    t1<T>(r: &mut T, x: T) {
        *r = x;
    }

    t2<T: resource>(r: &mut T, x: T) {
        *r = x;
    }

}

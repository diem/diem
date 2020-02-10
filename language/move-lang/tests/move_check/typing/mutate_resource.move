module M {
    resource struct R {}

    fun t0(r: &mut R) {
        *r = R {};
    }

    fun t1<T>(r: &mut T, x: T) {
        *r = x;
    }

    fun t2<T: resource>(r: &mut T, x: T) {
        *r = x;
    }

}

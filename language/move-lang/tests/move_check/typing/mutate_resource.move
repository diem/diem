module 0x8675309::M {
    struct R {}

    fun t0(r: &mut R) {
        *r = R {};
    }

    fun t1<T>(r: &mut T, x: T) {
        *r = x;
    }

    fun t2<T: key>(r: &mut T, x: T) {
        *r = x;
    }

}

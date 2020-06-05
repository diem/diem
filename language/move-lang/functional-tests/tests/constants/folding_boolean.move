script {
    const AND_TT: bool = true && true;
    const AND_TF: bool = true && false;
    const AND_FT: bool = false && true;
    const AND_FF: bool = false && false;

    const OR_TT: bool = true || true;
    const OR_TF: bool = true || false;
    const OR_FT: bool = false || true;
    const OR_FF: bool = false || false;

    const NEG_T: bool = !true;
    const NEG_F: bool = !false;

    const COMPLEX: bool = !((true && false) || (false || true) && true) || true;

    fun main() {
        assert(AND_TT, 42);
        assert(!AND_TF, 42);
        assert(!AND_FT, 42);
        assert(!AND_FF, 42);

        assert(OR_TT, 42);
        assert(OR_TF, 42);
        assert(OR_FT, 42);
        assert(!OR_FF, 42);

        assert(!NEG_T, 42);
        assert(NEG_F, 42);

        assert(COMPLEX, 42);
    }
}

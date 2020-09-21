script {
    const EQ_T_U8: bool = 0 == 0;
    const EQ_T_U64: bool = 0 == 0;
    const EQ_T_U128: bool = 0 == 0;
    const EQ_T_BOOL: bool = false == false;
    const EQ_T_ADDR: bool = 0x42 == 0x42;
    const EQ_T_HEX: bool = x"42" == x"42";
    const EQ_T_BYTES: bool = b"hello" == b"hello";

    const EQ_F_U8: bool = 0 == 1;
    const EQ_F_U64: bool = 0 == 1;
    const EQ_F_U128: bool = 0 == 1;
    const EQ_F_BOOL: bool = false == true;
    const EQ_F_ADDR: bool = 0x42 == 0x43;
    const EQ_F_HEX: bool = x"42" == x"422";
    const EQ_F_BYTES: bool = b"hello" == b"XhelloX";

    const NEQ_T_U8: bool = 0 != 1;
    const NEQ_T_U64: bool = 0 != 1;
    const NEQ_T_U128: bool = 0 != 1;
    const NEQ_T_BOOL: bool = false != true;
    const NEQ_T_ADDR: bool = 0x42 != 0x43;
    const NEQ_T_HEX: bool = x"42" != x"422";
    const NEQ_T_BYTES: bool = b"hello" != b"XhelloX";

    const NEQ_F_U8: bool = 0 != 0;
    const NEQ_F_U64: bool = 0 != 0;
    const NEQ_F_U128: bool = 0 != 0;
    const NEQ_F_BOOL: bool = false != false;
    const NEQ_F_ADDR: bool = 0x42 != 0x42;
    const NEQ_F_HEX: bool = x"42" != x"42";
    const NEQ_F_BYTES: bool = b"hello" != b"hello";

    const COMPLEX: bool = ((0 == 0) == (b"hello" != b"bye")) == false;

    fun main() {
        assert(EQ_T_U8, 42);
        assert(EQ_T_U64, 42);
        assert(EQ_T_U128, 42);
        assert(EQ_T_BOOL, 42);
        assert(EQ_T_ADDR, 42);
        assert(EQ_T_HEX, 42);
        assert(EQ_T_BYTES, 42);

        assert(!EQ_F_U8, 42);
        assert(!EQ_F_U64, 42);
        assert(!EQ_F_U128, 42);
        assert(!EQ_F_BOOL, 42);
        assert(!EQ_F_ADDR, 42);
        assert(!EQ_F_HEX, 42);
        assert(!EQ_F_BYTES, 42);

        assert(NEQ_T_U8, 42);
        assert(NEQ_T_U64, 42);
        assert(NEQ_T_U128, 42);
        assert(NEQ_T_BOOL, 42);
        assert(NEQ_T_ADDR, 42);
        assert(NEQ_T_HEX, 42);
        assert(NEQ_T_BYTES, 42);

        assert(!NEQ_F_U8, 42);
        assert(!NEQ_F_U64, 42);
        assert(!NEQ_F_U128, 42);
        assert(!NEQ_F_BOOL, 42);
        assert(!NEQ_F_ADDR, 42);
        assert(!NEQ_F_HEX, 42);
        assert(!NEQ_F_BYTES, 42);

        assert(!COMPLEX, 42);
    }
}

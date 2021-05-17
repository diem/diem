// Tests error reporting for use of old spec fun syntax.

module 0x8675309::M {

    spec fun with_aborts_if {
      aborts_if x == 0;
    }
    fun with_aborts_if(x: u64): u64 {
        x
    }

}

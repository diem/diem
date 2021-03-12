module 0x8675309::M {
    fun implies_in_prog(x: bool) {
      let _ = x ==> x;
    }
}

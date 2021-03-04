// flag: --v2
address 0x0 {
/// Example for opaque function calls.
module Trafo {
  public fun opaque_decr(x: u64): u64 {
      x - 1
  }
  spec fun opaque_decr {
      pragma opaque;
      aborts_if x == 0;
      ensures result == x - 1;
  }

  public fun opaque_caller(x: u64): u64 {
      opaque_decr(opaque_decr(x))
  }
  spec fun opaque_caller {
      aborts_if x < 2;
      ensures result == x - 2;
  }
}
}

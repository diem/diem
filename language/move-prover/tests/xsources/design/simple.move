// flag: --v2
address 0x0 {
/// Example for simple pre/post conditions.
module Trafo {

  public fun div(x: u64, y: u64): u64 {
      x / y
  }
  spec fun div {
      aborts_if y == 0;
      ensures result == x / y;
  }

}
}

// flag: --v2
address 0x0 {
/// Example for reference elimination.
module Trafo {

  struct R { x: u64 }

  public fun incr_ref(r: &mut u64) {
      *r = *r + 1;
  }
  spec fun incr_ref {
    ensures r == old(r) + 1;
  }

  public fun use_incr_ref(b: bool): R {
      let r1 = R{x:1};
      let r2 = R{x:2};
      let r_ref = if (b) &mut r1 else &mut r2;
      incr_ref(&mut r_ref.x);
      r2
  }
  spec fun use_incr_ref {
      ensures b ==> result == R{x:2};
      ensures !b ==> result == R{x:3};
  }

}
}

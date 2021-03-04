// flag: --v2
address 0x0 {
/// Example for opaque function calls with `modifies` clause.
module Trafo {

  resource struct R { x: u64 }

  public fun opaque_decr(addr: address) acquires R {
      let r = borrow_global_mut<R>(addr);
      r.x = r.x - 1;
  }
  spec fun opaque_decr {
      pragma opaque;
      modifies global<R>(addr);
      aborts_if !exists<R>(addr);
      aborts_if global<R>(addr).x == 0;
      ensures exists<R>(addr);
      ensures global<R>(addr).x == old(global<R>(addr).x) - 1;
  }

  public fun opaque_caller(addr: address) acquires R {
      opaque_decr(addr);
      opaque_decr(addr);
  }
  spec fun opaque_caller {
      modifies global<R>(addr);
      aborts_if !exists<R>(addr);
      aborts_if global<R>(addr).x < 2;
      ensures global<R>(addr).x == old(global<R>(addr).x) - 2;
  }
}
}

// flag: --v2
address 0x0 {
/// Example for resources.
module Trafo {
  use 0x1::Signer;

  resource struct R { x: u64 }

  public fun publish(account: &signer, x: u64) {
      move_to<R>(account, R{x: x})
  }
  spec fun publish {
      let addr = Signer::spec_address_of(account);
      aborts_if exists<R>(addr);
      ensures exists<R>(addr);
      ensures global<R>(addr).x == x;
  }

  public fun dummy_need_signer_use_in_move(account: &signer): address {
      // Move bug: we can't use Signer in specs if not also used in code.
      Signer::address_of(account)
  }
}
}

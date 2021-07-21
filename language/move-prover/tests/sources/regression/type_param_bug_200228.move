module 0x42::Test {

  struct Balance<phantom Token> has key {}
  struct EventHandle<phantom Token> has key {}

  fun type_param_bug<Tok_1: store, Tok_2: store>(addr: address): address {
    addr
  }
  spec type_param_bug {
    pragma verify=true;
    aborts_if false;
    ensures old(exists<Balance<Tok_1>>(addr)) ==> old(exists<Balance<Tok_2>>(addr)); // original bug: proved by Prover, but should not be.
  }
}

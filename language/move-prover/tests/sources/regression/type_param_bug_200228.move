module Test {

  resource struct Balance<Token> {}
  resource struct EventHandle<Token> {}

  fun type_param_bug<Tok_1, Tok_2>(addr: address): address {
    addr
  }
  spec fun type_param_bug {
    pragma verify=true;
    aborts_if false;
    ensures old(exists<Balance<Tok_1>>(addr)) ==> old(exists<Balance<Tok_2>>(addr)); // original bug: proved by Prover, but should not be.
  }
}

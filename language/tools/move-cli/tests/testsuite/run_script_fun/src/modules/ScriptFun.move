address 0x2 {
module ScriptFun {
  resource struct Called { i: u64 }

  public(script) fun script_fun(account: &signer, i: u64) {
      move_to(account, Called { i })
  }

  fun private_fun() {}

  public fun public_fun() {}
}
}

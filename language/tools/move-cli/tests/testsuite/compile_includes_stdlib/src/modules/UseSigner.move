module 0x1::Example {
  use 0x1::Signer;

  public fun f(account: &signer): address {
    Signer::address_of(account)
  }
}

address 0x42 {
module M {
  use 0x43::N;
  public fun foo() {
    N::bar()
  }
}
}

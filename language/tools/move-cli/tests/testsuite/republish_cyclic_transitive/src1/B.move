address 0x42 {
module B {
  use 0x42::C;
  public fun b() {
    C::c()
  }
}
}

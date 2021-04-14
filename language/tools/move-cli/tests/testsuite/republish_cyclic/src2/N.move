address 0x43 {
module N {
  use 0x42::M;
  public fun bar() {
    M::foo()
  }
}
}

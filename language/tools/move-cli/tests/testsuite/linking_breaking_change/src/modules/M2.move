address 0x2 {
module M2 {
  use 0x2::M;

  public fun h() {
      M::f();
      M::g();
  }
}
}

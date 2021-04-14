address 0x42 {
module A {
  use 0x42::B;
  public fun a() {
    B::b()
  }
}
}

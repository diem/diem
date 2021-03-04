address 0x42 {
module C {
  use 0x42::D;
  use 0x42::A;
  public fun c() {
    A::a();
    D::d()
  }
}
}

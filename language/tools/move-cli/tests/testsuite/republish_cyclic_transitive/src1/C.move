address 0x42 {
module C {
  use 0x42::D;
  public fun c() {
    D::d()
  }
}
}

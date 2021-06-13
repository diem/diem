module 0x42::M {

  fun invalid_old_exp(x: &mut u64, y: u64) {
    let a = x;
    let b = &mut y;
    *a = *b;
    *b = 0;
    spec {
      assert old(a) == y;
      assert old(b) == 0;
      assert old(x != y);
    }
  }
}

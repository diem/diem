module 0x42::M {
  fun foo(v: vector<u8>): vector<u8> {
    v
  }

  spec foo {
    ensures result == vec(1);
  }
}

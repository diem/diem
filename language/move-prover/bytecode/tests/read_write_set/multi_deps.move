address 0x1 {
module MultiDeps {
  struct S has key { f: u64 }
  struct T has key { f: u64 }

  fun add_to(s: &mut S, t: &T, v: bool) {
      *&mut s.f = if(v) { *&mut s.f } else { t.f }
  }

}
}

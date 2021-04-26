address 0x1 {
module Counter {
  struct S has key { f: u64 }

  fun increment1(i: &mut u64) {
      *i = *i + 1
  }

  fun increment2(s: &mut S) {
      *&mut s.f = *&mut s.f + 1
  }

  fun call_increment1(s: &mut S) {
      increment1(&mut s.f)
  }

  fun call_increment2(a: address) acquires S {
      increment2(borrow_global_mut<S>(a))
  }
}
}

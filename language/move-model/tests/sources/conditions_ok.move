module M {

  fun add_some(x: &mut u64): u64 { *x = *x + 1; *x }

  spec fun add_some {
    global some_global: u64;
    aborts_if x == 0 || some_global == 0;
    ensures old(x) > x;
    ensures result == x;
  }

  fun multiple_results(x: u64): (u64, bool) { (x, true) }

  spec fun multiple_results {
    ensures x == result_1 && result_2 == true;
  }

}

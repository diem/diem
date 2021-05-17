module 0x42::M {

  fun add_some(x: &mut u64): u64 { *x = *x + 1; *x }

  spec add_some {
    global some_global: u64;
    aborts_if x == 0 || some_global == 0;
    ensures old(x) > x;
    ensures result == x;
  }

  fun multiple_results(x: u64): (u64, bool) { (x, true) }

  spec multiple_results {
    ensures x == result_1 && result_2 == true;
  }

  fun with_emits<T: drop>(_guid: vector<u8>, _msg: T, x: u64): u64 { x }

  spec with_emits {
    emits _msg to _guid;
    emits _msg to _guid if true;
    emits _msg to _guid if x > 7;
  }
}

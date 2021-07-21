module 0x42::M {

  fun add_some(x: &mut u64): u64 { *x = *x + 1; *x }

  spec add_some {
    // Type of condition not bool.
    aborts_if x;
    ensures old(x) + x;
    // Using result which does not exist.
    ensures result_1 == 0;
    // Type parameter not allowed
    aborts_with<T: drop> 42;
    modifies<T: key> global<T>(0x1);
  }

  fun with_emits<T: drop>(_guid: vector<u8>, _msg: T, x: u64): u64 { x }

  spec with_emits {
    // Type of condition for "if" is not bool.
    emits _msg to _guid if x;
    // Type parameter not allowed
    emits<T: drop> _msg to _guid;
  }
}

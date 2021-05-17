module 0x42::M {

  fun foo(x: &mut u64): u64 { *x = *x + 1; *x }

  spec foo {
    let zero = one;
    ensures result == old(x) + 1;
  }

  fun spec_let_with_schema(a: &mut u64, b: &mut u64) {
    let saved_a = *a;
    *a = *a / (*a + *b);
    *b = saved_a * *b;
  }
  spec spec_let_with_schema {
    let sum = a + b;
    let product = a * b;
    aborts_if sum == 0;
    aborts_if sum > MAX_U64;
    aborts_if product > MAX_U64;
    let post new_a = old(a) / sum;
    include Ensures{actual: a, expected: new_a + sum - sum};
    include Ensures{actual: b, expected: product};
  }
  spec schema Ensures {
    actual: u64;
    expected: u64;
    let a = expected;
    let b = actual;
    include Ensures2{a: a, b: b};
  }
  spec schema Ensures2 {
    a: u64;
    b: u64;
    ensures a == b;
  }

  fun result_with_schema(x: &mut u64): u64 {
    *x = 2;
    *x
  }
  spec result_with_schema {
    include Requires{a: result};
    include Requires{a: old(x)};
  }
  spec schema Requires {
    a: u64;
    requires a != 0;
  }
}

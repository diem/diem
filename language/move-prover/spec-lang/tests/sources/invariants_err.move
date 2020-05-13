module M {

  resource struct S {
    x: u64,
  }

  spec struct S {
    // Expression not a bool
    invariant x + 1;
    // Old expression in data invariant
    invariant old(x) > 0;
    // Nested old expression.
    // invariant update old(old(x)) > 0;
    // Direct dependency from global state
    invariant exists<S>(0x0);
    invariant global<S>(0x0).x == x;
    invariant sender() == 0x0;
    invariant spec_var > 0;
    // Indirect dependency from global state via function call.
    invariant rec_fun(true);
  }

  spec module {
    global spec_var: num;

    define rec_fun(c: bool): bool {
        if (c) {
          rec_fun2(c)
        } else {
          spec_var > 0
        }
      }
      define rec_fun2(c: bool): bool {
         rec_fun(!c)
      }
    }
}

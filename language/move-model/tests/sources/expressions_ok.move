module 0x42::M {

  spec module {

    fun add1(x: num): num { x + 1 }

    fun call_other(x: num): num {
      add1(x)
    }

    fun call_other_self(x: num): num {
      Self::add1(x)
    }

    fun add_any_unsigned(x: u64, y: u8): num {
      x + y
    }

    fun compare_any_unsigned(x: u64, y: u128, z: num): bool {
      x == y && y == z ==> x == z
    }

    fun some_range(upper: num): range {
      0..upper
    }

    fun add_with_let(x: num, y: num): num {
      let r = x + y;
      r
    }

    /* Produces an error as we have disallowed shadowing for now.
     * TODO(wrwg): reactivate once we allow shadowing again
    fun let_shadows(): num {
      let x = true;
      let b = !x;
      let x = 1;
      x
    }
    */

    fun lambdas(p1: |num|bool, p2: |num|bool): |num|bool {
      |x| p1(x) && p2(x)
    }

    fun call_lambdas(x: num): bool {
      let f = lambdas(|y| y > 0, |y| y < 10);
      f(x)
    }

    fun if_else(x: num, y: num): num {
      if (x > 0) { x } else { y }
    }

    fun vector_builtins(v: vector<num>): bool {
      len(v) > 2 && (forall x in v: x > 0) && (exists x in v: x > 10) && update(v, 2, 23)[2] == 23
    }

    fun range_builtins(v: vector<num>): bool {
      (forall x in 1..10: x > 0) && (exists x in 5..10: x > 7)
    }

    fun vector_index(v: vector<num>): num {
      v[2]
    }

    fun vector_slice(v: vector<num>): vector<num> {
      v[2..3]
    }

    fun generic_function<T>(x: T, y: T): bool {
      x == y
    }
    fun generic_function_call(): bool {
      generic_function(1, 2)
    }
    fun generic_function_instantiated_call(): bool {
      generic_function<num>(3, 3)
    }

  }
}

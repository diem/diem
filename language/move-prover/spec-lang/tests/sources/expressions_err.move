module M {

  struct S {
    x: u64,
    y: bool,
  }

  spec module {

    // Undeclared simple name.
    define undeclared_name() : num {
      x
    }

    // Undeclared spec function.
    define undeclared_fun(): num {
      not_declared()
    }

    // Wrong result type.
    define wrong_result_type(): num {
      false
    }

    // No matching overload.
    define no_overload(x: vector<num>, y: vector<num>): bool {
      x > y
    }

    // Wrong result type tuple.
    define wrong_result_type(): (num, bool) {
      false
    }

    // Wrongly typed function application.
    define wrongly_typed_callee(x: num, y: bool): num { x }
    define wrongly_typed_caller(): num { wrongly_typed_callee(1, 1) }

    // Wrongly typed function argument.
    define wrongly_typed_fun_arg_callee(f: |num|num): num { 0 }
    define wrongly_typed_fun_arg_caller(): num { wrongly_typed_fun_arg_callee(|x| false) }

    // Ambiguous application
    // TODO: we actually want to see the error at the declaration, but currently we see it at the call.
    define ambigous_callee(): num { 0 }
    define ambigous_callee(): num { 0 }
    define amdbigous_caller(): num {
      ambigous_callee()
    }

    // Wrong instantiation
    define wrong_instantiation<T1, T2>(x: T1): T1 { x }
    define wrong_instantiation_caller(x: u64): u64 {
      wrong_instantiation<u64>(x)
    }
  }
}

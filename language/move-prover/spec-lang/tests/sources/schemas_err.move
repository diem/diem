module M {

    spec schema UndeclaredVar {
        ensures x > 0;
    }

    spec schema UndeclaredSchema {
        include Undeclared;
    }

    spec schema WrongTypeArgs {
        include WrongTypeArgsIncluded<num, num>;
    }
    spec schema WrongTypeArgsIncluded<T> {
        x: T;
    }

    spec schema WrongRenaming {
        include WrongTypeArgsIncluded<num>{wrong: 1};
    }

    spec schema WrongTypeAfterRenaming {
        y: bool;
        include WrongTypeArgsIncluded<num>{x: y};
    }

    spec schema WrongTypeAfterRenamingExp {
        include WrongTypeArgsIncluded<bool>{x: 1 + 2};
    }

    spec schema WronglyTypedVar {
        x: bool;
        include WronglyTypedVarIncluded;
    }
    spec schema WronglyTypedVarIncluded {
        x: num;
    }

    spec schema WronglyTypedInstantiation {
        x: bool;
        include WronglyTypedInstantiationIncluded<num>;
    }
    spec schema WronglyTypedInstantiationIncluded<T> {
        x: T;
    }

    fun undeclared_var_in_include(x: u64): u64 { x }
    spec schema UndeclaredVarInInclude {
        y: u64;
    }
    spec fun undeclared_var_in_include {
        include UndeclaredVarInInclude;
    }

    fun wrong_invariant_in_fun(x: u64): u64 { x }
    spec schema Invariant {
        x: num;
        invariant x > 0;
    }
    spec fun wrong_invariant_in_fun {
        include Invariant;
    }

    struct wrong_condition_in_struct { x: u64 }
    spec schema Condition {
        x: num;
        requires x > 0;
    }
    spec struct wrong_condition_in_struct {
        include Condition;
    }

    spec schema Cycle1 {
        include Cycle2;
    }
    spec schema Cycle2 {
        include Cycle3;
    }
    spec schema Cycle3 {
        include Cycle1;
    }
}

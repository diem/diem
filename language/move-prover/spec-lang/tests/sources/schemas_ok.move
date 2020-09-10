module M {

    spec schema Increases {
        x: num;
        result: num;
        requires x > 0;
        ensures result >= x;
    }

    spec schema IncreasesStrictly {
        include Increases;
        ensures result > x;
    }

    spec schema IncreasesWithTwoResults {
        result_1: num;
        result_2: num;
        include Increases{result: result_1};
        ensures result_2 > result_1;
    }

    spec schema IsEqual<T> {
        x: T;
        y: T;
        ensures x == y;
    }

    spec schema IsEqualConcrete {
        z: num;
        include IsEqual<num>{x: z};
        ensures z <= y;
    }

    fun add(x: u64): u64 { x + 1 }
    spec fun add {
        include Increases;
    }

    fun id(x: u64): u64 { x }
    spec fun add {
        include IsEqual<num>{y: result};
    }

    spec schema InvariantIsEqual<T> {
        x: T;
        // invariant update x == old(x);
    }
    struct S<X> { x: X }
    spec struct S {
        include InvariantIsEqual<X>;
    }

    spec schema GenericIncludesGeneric<T> {
        include InvariantIsEqual<T>;
    }

    spec schema MultipleTypeParams<T, R> {
        _x: T;
        _y: R;
    }
    fun multiple(_x: u64, _y: u64) {}
    spec fun multiple {
        include MultipleTypeParams<num, num>;
        requires _x > _y;
    }

    spec schema SchemaExp<T> {
        x: bool;
        include x ==> InvariantIsEqual<bool>;
        include !x ==> InvariantIsEqual<bool>;
        include InvariantIsEqual<bool> && InvariantIsEqual<bool>;
        include if (x) InvariantIsEqual<bool> else InvariantIsEqual<bool>;
    }

}

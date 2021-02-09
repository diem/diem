module TestSchemaExp {

    spec module {
        pragma verify = true;
    }


    spec schema Aborts {
        aborts_if true;
    }

    spec schema DontAborts {
        aborts_if false;
    }

    fun foo(c: bool) {
        if (!c) abort(1);
    }

    spec fun foo {
        include c ==> DontAborts;
        include !c ==> Aborts;
    }

    fun bar_incorrect(c: bool) {
        if (!c) abort(1);
    }

    spec fun bar_incorrect {
        // Once we include a schema with aborts, even conditionally, we need to provide a full spec of the aborts
        // behavior. This is because the below translates to `aborts_if c && false`, which reduces
        // to `aborts_if false`.
        include c ==> DontAborts;
    }

    fun baz(i: u64): u64 {
        if (i > 10) { i + 2 } else { i + 1 }
    }
    spec schema AddsOne {
        i: num;
        result: num;
        ensures result == i + 1;
    }
    spec schema AddsTwo {
        i: num;
        result: num;
        ensures result == i + 2;
    }
    spec fun baz {
        include if (i > 10) AddsTwo else AddsOne;
    }

    fun baz_incorrect(i: u64): u64 {
        i + 1
    }
    spec fun baz_incorrect {
        include i > 10 ==> AddsTwo;
        include i <= 10 ==> AddsOne;
    }

}

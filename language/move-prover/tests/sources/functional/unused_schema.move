// flag: --severity=note
module 0x42::TestUnusedSchema {

    spec module {
        pragma verify = true;
    }

    spec schema AddsOne {
        i: num;
        result: num;
        ensures result >= i + 1;
    }

    spec schema AddsTwo {
        i: num;
        result: num;
        ensures result == i + 2;
        include AddsOne;
    }

    // AddsThree is the only unused schema
    spec schema AddsThree {
        i: num;
        result: num;
        ensures result == i + 3;
    }

    spec schema UNUSED_AddsFour {
        i: num;
        result: num;
        ensures result == i + 4;
    }

    fun foo(i: u64): u64 {
        if (i > 10) { i + 2 } else { i + 1 }
    }

    spec foo {
        include i > 10 ==> AddsTwo;
    }
}

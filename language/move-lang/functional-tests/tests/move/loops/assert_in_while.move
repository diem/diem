module Test {
    struct Tester has drop {
        f: u64
    }

    public fun new(): Tester {
        Tester { f: 10 }
    }

    public fun len(t: &Tester): u64 {
        t.f
    }

    public fun modify(t: &mut Tester): u64 {
        t.f = t.f - 1;
        9 - t.f
    }
}

//! new-transaction

script {
use {{default}}::Test;
fun main() {
    let x = Test::new();
    assert(Test::len(&x) == 10, 70002);

    let i = 0;
    while (i < Test::len(&x)) {
        // if inline blocks skips relabelling this will cause a bytecode verifier error
        assert(Test::modify(&mut x) == i, 70003);
        i = i + 1
    }
}
}

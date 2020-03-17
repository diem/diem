module TestDefines {

    spec module {
        define in_range(x: num, min: num, max: num): bool {
            x >= min && x <= max
        }

        define eq<T>(x: T, y: T): bool {
            x == y
        }
    }

    fun add(x: u64, y: u64): u64 { x + y }

    spec fun add {
        aborts_if !in_range(x + y, 0, 18446744073709551615);
        ensures eq(result, x + y);
    }
}

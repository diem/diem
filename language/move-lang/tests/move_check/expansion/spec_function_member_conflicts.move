address 0x1 {

module X1 {
    spec module {
        define Foo(): bool { true }
    }
    struct Foo {}
}

module X2 {
    spec module {
        define Foo(): bool { true }
    }
    spec schema Foo<T> {
        ensures true;
    }
}

module X3 {
    spec module {
        define Foo(): bool { true }
    }
    fun Foo() {}
}

module X4 {
    spec module {
        define foo(): bool { true }
    }
    fun foo() {}
}

module X5 {
    spec module {
        define foo(): bool { true }
    }
    spec module {
        define foo(): bool { true }
    }
}
}

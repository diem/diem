address 0x1 {
    module X {
        struct S {}
        public fun foo() {}
    }

    module M {
        use 0x1::X::{Self as B, foo, S};

        struct X { f: B::S, f2: S }
        fun bar() {
            B::foo();
            foo()
        }
    }
    }

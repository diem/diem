address 0x42 {
    module M {
        struct Option { f: u64 }
        public fun ex(): Option {
            Option { f: 0 }
        }
    }

    module N {
        use 0x42::M as Option;
        use 0x42::M as Signer;
        use 0x42::M as Vector;

        fun t() {
            Option::ex();
            Signer::ex();
            Vector::ex();
        }

    }
}

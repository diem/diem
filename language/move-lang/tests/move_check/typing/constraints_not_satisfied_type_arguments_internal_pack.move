module 0x8675309::M {
    struct CupD<T: drop> has drop { f: T }
    struct R {}

    struct Box<T> has drop { f: T }

    fun foo() {
        Box<CupD<R>>{ f: abort 0 };
        Box<R>{ f: R{} };
    }

}

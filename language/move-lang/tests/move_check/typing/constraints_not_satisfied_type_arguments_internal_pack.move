module M {
    struct CupD<T: drop> has drop {}
    struct R {}

    struct Box<T> has drop {}

    fun foo() {
        Box<CupD<R>>{};
        Box<R>{};
    }

}

module 0x2::Token {
    struct Token<phantom T> has store { value: u64 }

    public fun burn<T>(_unused: bool, token: Token<T>) {
        let Token { value } = token;
        assert(value != 0, 42);
    }
    spec burn {
        aborts_if token.value == 0;
    }
}

module 0x2::Liquid {
    use 0x2::Token;

    struct Liquid<phantom X, phantom Y> has key, store {}

    fun l_burn<X, Y>(token: Token::Token<Liquid<X, Y>>) {
        Token::burn(false, token);
    }
}

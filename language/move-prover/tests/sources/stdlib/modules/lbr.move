address 0x0 {

module LBR {
    use 0x0::Transaction;
    use 0x0::Libra;

    struct T { }

    public fun initialize() {
        Transaction::assert(Transaction::sender() == 0xA550C18, 0);
        Libra::register<T>();
    }
    spec fun initialize {
        // These conditions ensure that ghost variables are properly initialized
        // before "initialize" is called. Otherwise, "Libra::register" will fail
        // because they are ensures conditions of "Libra::register"
        requires Libra::mint_capability_count<T> == 0;
        requires Libra::sum_of_token_values<T> == 0;

        // FIXME:
        // The next is actually a conceptual problem in the prover. "initialize"
        // registers the LBR token type, so it is only useful at the beginning
        // before the LBR is registered. However, if initialize is erroneously called
        // a second time, it will abort in Libra::register<T>, because it will do a move_to_sender
        // to the association account, which will abort if the Info<Token> resource
        // is already there (that's the test for whether a token is registered).

        // However, the invariant adds a "requires"
        // token_is_registered<Token>() ==> mint_capability_count<Token> == 1;
        // and this fails because it doesn't know the function is going to abort.
        // Does aborts_if P effectively imply requires !P?
        requires !Libra::token_is_registered<T>();

        // FIXME:
        // Another bug:
        // Boogie reports:
        // bug: output.bpl(3882,158): Error: undeclared identifier: $tv0
        // include Libra::RegisterAbortsIf<T>;
    }
}
}

address 0x0:

module Empty {
    // An empty account cannot hold money, nor can it send or receive
    // money. This is why it doesn't need an AccountLimits::Window to hold
    // tracking information.
    struct T { }

    public fun create(): T {
        T { }
    }
}

module 0x1::Token {
    use Std::Signer;
    use Std::Errors;
    use 0x42::Limits;

    /// Errors
    const INSUFFICIENT_BALANCE: u64 = 1;
    const BALANCE_OVERFLOW: u64 = 2;
    const SEND_TO_SELF: u64 = 3;

    struct Mint<Meta: copy + drop + store> has key, store {
        meta: Meta,
        total_supply: u64,
        // name: vector<u8>,
        // symbol: vector<u8>,
        // scaling_factor: u64,
        // fractional_part: u64,
        // can_mint: bool,
        // mint_event: EventHandle<MintEvent>,
        // burn_event: EventHandle<BurnEvent>,
    }

    struct Account<phantom Meta> has key, store {
        balance: u64,
        // sent_event: EventHandle<SentEvent>,
        // received_event: EventHandle<ReceivedEvent>,
    }

    public fun total_supply<Meta: copy + drop + store>(minter: address): u64 acquires Mint {
        borrow_global<Mint<Meta>>(minter).total_supply
    }

    public fun meta<Meta: copy + drop + store>(minter: address): Meta acquires Mint {
        *&borrow_global<Mint<Meta>>(minter).meta
    }

    public fun balance<Meta: copy + drop + store>(account: address): u64 acquires Account {
        borrow_global<Account<Meta>>(account).balance
    }

    public(script) fun publish<Meta: copy + drop + store>(minter: &signer, meta: Meta, total_supply: u64) {
        move_to(minter, Mint<Meta> { total_supply, meta });
        move_to(minter, Account<Meta> { balance: total_supply });
    }

    public(script) fun create_account<Meta>(owner: &signer) {
        move_to(owner, Account<Meta> { balance: 0 });
    }

    fun withdraw<Meta: copy + drop + store>(account: address, amount: u64) acquires Account {
        let balance = &mut borrow_global_mut<Account<Meta>>(account).balance;
        assert(*balance >= amount, Errors::limit_exceeded(INSUFFICIENT_BALANCE));
        *balance = *balance - amount;
    }

    fun deposit<Meta: copy + drop + store>(account: address, amount: u64) acquires Account {
        let balance = &mut borrow_global_mut<Account<Meta>>(account).balance;
        assert(*balance <= Limits::max_u64() - amount, Errors::limit_exceeded(BALANCE_OVERFLOW));
        *balance = *balance + amount;
    }

    public(script) fun transfer<Meta: copy + drop + store>(from: &signer, to: address, amount: u64) acquires Account {
        assert(Signer::address_of(from) != to, Errors::invalid_argument(SEND_TO_SELF));
        withdraw<Meta>(Signer::address_of(from), amount);
        deposit<Meta>(to, amount);
    }
}

module 0x42::Limits {
    public fun max_u64(): u64 { 18446744073709551615 }
}

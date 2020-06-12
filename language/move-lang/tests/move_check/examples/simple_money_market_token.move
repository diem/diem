address 0x1 {

module Token {

    resource struct Coin<AssetType: copyable> {
        type: AssetType,
        value: u64,
    }

    // control the minting/creation in the defining module of `ATy`
    public fun create<ATy: copyable>(type: ATy, value: u64): Coin<ATy> {
        Coin { type, value }
    }

    public fun value<ATy: copyable>(coin: &Coin<ATy>): u64 {
        coin.value
    }

    public fun split<ATy: copyable>(coin: Coin<ATy>, amount: u64): (Coin<ATy>, Coin<ATy>) {
        let other = withdraw(&mut coin, amount);
        (coin, other)
    }

    public fun withdraw<ATy: copyable>(coin: &mut Coin<ATy>, amount: u64): Coin<ATy> {
        assert(coin.value >= amount, 10);
        coin.value = coin.value - amount;
        Coin { type: *&coin.type, value: amount }
    }

    public fun join<ATy: copyable>(coin1: Coin<ATy>, coin2: Coin<ATy>): Coin<ATy> {
        deposit(&mut coin1, coin2);
        coin1
    }

    public fun deposit<ATy: copyable>(coin: &mut Coin<ATy>, check: Coin<ATy>) {
        let Coin { value, type } = check;
        assert(&coin.type == &type, 42);
        coin.value = coin.value + value;
    }

    public fun destroy_zero<ATy: copyable>(coin: Coin<ATy>) {
        let Coin { value, type: _ } = coin;
        assert(value == 0, 11)
    }

}

}

address 0xB055 {

module OneToOneMarket {
    use 0x0::Signer;
    use 0x1::Token;

    resource struct Pool<AssetType: copyable> {
        coin: Token::Coin<AssetType>,
    }

    resource struct DepositRecord<InputAsset: copyable, OutputAsset: copyable> {
        record: u64,
    }

    resource struct BorrowRecord<InputAsset: copyable, OutputAsset: copyable> {
        record: u64,
    }

    resource struct Price<InputAsset: copyable, OutputAsset: copyable> {
        price: u64,
    }

    fun accept<AssetType: copyable>(account: &signer, init: Token::Coin<AssetType>) {
        let sender = Signer::address_of(account);
        assert(!exists<Pool<AssetType>>(sender), 42);
        move_to(account, Pool<AssetType> { coin: init })
    }

    public fun register_price<In: copyable, Out: copyable>(
        account: &signer,
        initial_in: Token::Coin<In>,
        initial_out: Token::Coin<Out>,
        price: u64
    ) {
        let sender = Signer::address_of(account);
        assert(sender == 0xB055, 42); // assert sender is module writer
        accept<In>(account, initial_in);
        accept<Out>(account, initial_out);
        move_to(account, Price<In, Out> { price })
    }

    public fun deposit<In: copyable, Out: copyable>(account: &signer, coin: Token::Coin<In>)
        acquires Pool, DepositRecord
    {
        let amount = Token::value(&coin);

        update_deposit_record<In, Out>(account, amount);

        let pool = borrow_global_mut<Pool<In>>(0xB055);
        Token::deposit(&mut pool.coin, coin)
    }

    public fun borrow<In: copyable, Out: copyable>(
        account: &signer,
        amount: u64,
    ): Token::Coin<Out>
        acquires Price, Pool, DepositRecord, BorrowRecord
    {
        assert(amount <= max_borrow_amount<In, Out>(account), 1025);

        update_borrow_record<In, Out>(account, amount);

        let pool = borrow_global_mut<Pool<Out>>(0xB055);
        Token::withdraw(&mut pool.coin, amount)
    }

    fun max_borrow_amount<In: copyable, Out: copyable>(account: &signer): u64
        acquires Price, Pool, DepositRecord, BorrowRecord
    {
        let input_deposited = deposited_amount<In, Out>(account);
        let output_deposited = borrowed_amount<In, Out>(account);

        let input_into_output =
            input_deposited * borrow_global<Price<In, Out>>(0xB055).price;
        let max_output =
            if (input_into_output < output_deposited) 0
            else (input_into_output - output_deposited);
        let available_output = {
            let pool = borrow_global<Pool<Out>>(0xB055);
            Token::value(&pool.coin)
        };
        if (max_output < available_output) max_output else available_output

    }

    fun update_deposit_record<In: copyable, Out: copyable>(account: &signer, amount: u64)
        acquires DepositRecord
    {
        let sender = Signer::address_of(account);
        if (!exists<DepositRecord<In, Out>>(sender)) {
            move_to(account, DepositRecord<In, Out> { record: 0 })
        };
        let record = &mut borrow_global_mut<DepositRecord<In, Out>>(sender).record;
        *record = *record + amount
    }

    fun update_borrow_record<In: copyable, Out: copyable>(account: &signer, amount: u64)
        acquires BorrowRecord
    {
        let sender = Signer::address_of(account);
        if (!exists<BorrowRecord<In, Out>>(sender)) {
            move_to(account, BorrowRecord<In, Out> { record: 0 })
        };
        let record = &mut borrow_global_mut<BorrowRecord<In, Out>>(sender).record;
        *record = *record + amount
    }

    fun deposited_amount<In: copyable, Out: copyable>(account: &signer): u64
        acquires DepositRecord
    {
        let sender = Signer::address_of(account);
        if (!exists<DepositRecord<In, Out>>(sender)) return 0;
        borrow_global<DepositRecord<In, Out>>(sender).record
    }

    fun borrowed_amount<In: copyable, Out: copyable>(account: &signer): u64
        acquires BorrowRecord
    {
        let sender = Signer::address_of(account);
        if (!exists<BorrowRecord<In, Out>>(sender)) return 0;
        borrow_global<BorrowRecord<In, Out>>(sender).record
    }
}

}

address 0x70DD {

module ToddNickles {
    use 0x1::Token;
    use 0x0::Signer;

    struct T {}

    resource struct Wallet {
        nickles: Token::Coin<T>,
    }

    public fun init(account: &signer) {
        assert(Signer::address_of(account) == 0x70DD, 42);
        move_to(account, Wallet { nickles: Token::create(T{}, 0) })
    }

    public fun mint(account: &signer): Token::Coin<T> {
        assert(Signer::address_of(account) == 0x70DD, 42);
        Token::create(T{}, 5)
    }

    public fun destroy(c: Token::Coin<T>) acquires Wallet {
        Token::deposit(&mut borrow_global_mut<Wallet>(0x70DD).nickles, c)
    }

}

}

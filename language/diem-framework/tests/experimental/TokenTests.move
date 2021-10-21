#[test_only]
module 0x101::MyCoin{
    use Std::Signer;

    /// A test token type to instantiate generic Tokens with.
    struct MyCoin has copy, drop, store {
        notes: vector<u8>,
    }

    public(script) fun new(me: &signer, notes: vector<u8>): MyCoin {
        assert(Signer::address_of(me) == @0x101, 1);
        MyCoin {notes}
    }

    public fun notes(meta: &MyCoin): &vector<u8> {
        &meta.notes
    }
}

#[test_only]
module 0x1::TokenTests {
    use 0x101::MyCoin;
    use 0x101::MyCoin::MyCoin;
    use 0x1::Token;

    const MINTER: address = @0x101;
    const USER1: address = @0x102;
    const USER2: address = @0x103;

    #[test(minter=@0x101, user1=@0x102, user2=@0x103)]
    public(script) fun test_happy_path(minter: signer, user1: signer, user2: signer) {
        Token::publish(&minter, MyCoin::new(&minter, b"My social coin."), 10000);
        assert(Token::total_supply<MyCoin>(MINTER) == 10000, 1);
        assert(Token::balance<MyCoin>(MINTER) == 10000, 1);
        assert(*MyCoin::notes(&Token::meta<MyCoin>(MINTER)) == b"My social coin.", 1);

        Token::create_account<MyCoin>(&user1);
        assert(Token::balance<MyCoin>(USER1) == 0, 1);
        Token::create_account<MyCoin>(&user2);
        assert(Token::balance<MyCoin>(USER2) == 0, 1);

        Token::transfer<MyCoin>(&minter, USER1, 500);
        Token::transfer<MyCoin>(&minter, USER2, 500);
        Token::transfer<MyCoin>(&user1, USER2, 200);
        assert(Token::balance<MyCoin>(MINTER) == 9000, 1);
        assert(Token::balance<MyCoin>(USER1) == 300, 1);
        assert(Token::balance<MyCoin>(USER2) == 700, 1);
    }
}

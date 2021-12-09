module Sender::NFTSale {
    use 0x1::NFT;
    use 0x1::NFTGallery;
    use Std::FixedPoint32;
    use Std::GUID;
    use Std::Option;
    use Std::Signer;
    use Sender::BasicCoin;

    /// Sell `amount` of nft tokens from `seller` to `buyer` at `price`, paid for using `CoinType`
    public fun sell<NFTType: store, CoinType: copy + drop>(
        seller: &signer, buyer: &signer,
        token_id: &GUID::ID,
        amount: u64, price: u64, witness: CoinType
    ) {
        let buyer_addr = Signer::address_of(buyer);
        let seller_addr = Signer::address_of(seller);
        let creator_addr = GUID::id_creator_address(token_id);

        // get royalty rate of token
        let royalty_rate_opt = NFT::royalty_rate<NFTType>(token_id);
        let seller_profit = price;

        // distribte royalties to creator
        if (Option::is_some(&royalty_rate_opt)) {
            let royalty_rate = Option::extract(&mut royalty_rate_opt);
            let royalty = FixedPoint32::multiply_u64(price, royalty_rate);
            seller_profit = price - royalty;
            BasicCoin::transfer<CoinType>(buyer_addr, creator_addr, royalty, copy witness);
        };

        // coin transfer from buyer to seller
        BasicCoin::transfer<CoinType>(buyer_addr, seller_addr, seller_profit, witness);

        // nft transfer from seller to buyer
        NFTGallery::transfer<NFTType>(seller, buyer_addr, amount, token_id);
    }


    /*
        Unit Test
    */

    struct TestCoin has copy, drop {}

    struct TestNFT has store {}

    #[test(creator=@0xA, seller=@0xB, buyer=@0xC, admin=@DiemRoot)]
    public(script) fun test_sale_royalty(creator: signer, buyer: signer, seller: signer, admin: signer) {
        let creator_addr = @0xA;
        let seller_addr = @0xB;
        let buyer_addr = @0xC;

        // initialize and set up resources
        NFT::nft_initialize(admin);
        BasicCoin::publish_balance<TestCoin>(&creator);
        BasicCoin::publish_balance<TestCoin>(&buyer);
        BasicCoin::publish_balance<TestCoin>(&seller);
        NFTGallery::publish_gallery<TestNFT>(&creator);
        NFTGallery::publish_gallery<TestNFT>(&buyer);
        NFTGallery::publish_gallery<TestNFT>(&seller);

        // mint 100 TestCoins to buyer
        BasicCoin::mint<TestCoin>(buyer_addr, 100, TestCoin {});

        // create 10 TestNFTs and add them to seller's gallery
        let nft = NFT::create(
            &creator,
            TestNFT {},
            b"",
            10,
            Option::none(),
            Option::some(FixedPoint32::create_from_rational(1, 10))
        );

        NFTGallery::add_to_gallery(seller_addr, nft);

        let token_id = GUID::create_id(creator_addr, 0);

        // sell
        sell<TestNFT, TestCoin>(&seller, &buyer, &token_id, 10, 42, TestCoin {});

        assert!(BasicCoin::balance_of<TestCoin>(creator_addr) == 4, 0);
        assert!(BasicCoin::balance_of<TestCoin>(seller_addr) == 38, 0);
        assert!(BasicCoin::balance_of<TestCoin>(buyer_addr) == 58, 0);
    }
}
module 0x1::BARSToken {
    use Std::Option;
    use Std::Signer;
    use 0x1::NFT;
    use 0x1::NFTGallery;
    #[test_only]
    use Std::GUID;

    struct BARSToken has store {
        artist_name: vector<u8>
    }

    /// Call this function to set up relevant resources in order to
    /// mint and receive tokens.
    public(script) fun register_user(user: signer) {
        register_user_internal(&user);
    }

    /// Need this internal function for testing, since the script fun version
    /// consumes a signer
    fun register_user_internal(user: &signer) {
        // publish TokenBalance<BARSToken> resource
        NFTGallery::publish_gallery<BARSToken>(user);

        // publish TokenDataCollection<BARSToken> resource
        NFT::publish_token_data_collection<BARSToken>(user);
    }

    /// Mint `amount` copies of BARS tokens to the artist's account.
    public(script) fun mint_bars(
        artist: signer,
        artist_name: vector<u8>,
        content_uri: vector<u8>,
        amount: u64
    ) {
        mint_internal(&artist, artist_name, content_uri, amount);
    }

    /// Need this internal function for testing, since the script fun version
    /// consumes a signer
    fun mint_internal(
        artist: &signer,
        artist_name: vector<u8>,
        content_uri: vector<u8>,
        amount: u64
    ) {
        let token = NFT::create<BARSToken>(
            artist,
            BARSToken { artist_name },
            content_uri,
            amount,
            Option::none(),
        );
        NFTGallery::add_to_gallery(Signer::address_of(artist), token);
    }

    #[test_only]
    const EMINT_FAILED: u64 = 0;
    #[test_only]
    const ETRANSFER_FAILED: u64 = 1;

    #[test(admin=@DiemRoot, artist=@0x42, collector=@0x43)]
    public(script) fun test_bars(admin: signer, artist: signer, collector: signer) {
        NFT::nft_initialize(admin);

        register_user_internal(&artist);
        register_user_internal(&collector);

        let token_id = GUID::create_id(@0x42, 0);
        mint_internal(&artist, b"kanye", b"yeezy.com", 7);

        assert(NFTGallery::has_token<BARSToken>(@0x42, &token_id), EMINT_FAILED);
        assert(NFTGallery::get_token_balance<BARSToken>(@0x42, &token_id) == 7, EMINT_FAILED);
        assert(NFT::supply<BARSToken>(&token_id) == 7, EMINT_FAILED);


        // Transfer 6 units of the token from creator to user
        NFTGallery::transfer_token_between_galleries<BARSToken>(
            artist, // from
            Signer::address_of(&collector), // to
            6, // amount
            @0x42, // token.id.addr
            0, // token.id.creation_num
        );
        assert(NFTGallery::get_token_balance<BARSToken>(@0x42, &token_id) == 1, ETRANSFER_FAILED);
        assert(NFTGallery::get_token_balance<BARSToken>(@0x43, &token_id) == 6, ETRANSFER_FAILED);
    }

}

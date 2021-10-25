module 0x1::BARSToken {
    use Std::Option;
    #[test_only]
    use Std::Signer;
    use 0x1::NFT;
    use 0x1::NFTGallery;
    #[test_only]
    use Std::GUID;

    const BARS: address = @0x34CD440B72D03907100007D7BC0080A1;

    // Error codes
    /// Function can only be called by the module owner
    const ENOT_BARS_OWNER: u64  = 0;

    struct BARSToken has store {
        artist_name: vector<u8>
    }

    /// Call this function to set up relevant resources in order to
    /// mint and receive tokens.
    /// Note that this also gives BARS account a capability to mint BARS NFTs on behalf of the user.
    /// (the NFTs of other types cannot be created by BARS account).
    public(script) fun register_bars_user(user: signer) {
        register_user_internal(&user);
    }

    /// Need this internal function for testing, since the script fun version
    /// consumes a signer
    fun register_user_internal(user: &signer) {
        // publish TokenBalance<BARSToken> resource
        NFTGallery::publish_gallery<BARSToken>(user);

        // publish TokenDataCollection<BARSToken> resource
        NFT::publish_token_data_collection<BARSToken>(user);

        // The user gives BARS account capability to generate BARS NFTs on their behalf.
        NFT::allow_creation_delegation<BARSToken>(user);
    }

    /// BARS account mints `amount` copies of BARS tokens to the artist's account.
    public(script) fun mint_bars(
        bars_account: signer,
        artist: address,
        artist_name: vector<u8>,
        content_uri: vector<u8>,
        amount: u64
    ) {
        mint_internal(&bars_account, artist, artist_name, content_uri, amount);
    }

    /// Need this internal function for testing, since the script fun version
    /// consumes a signer
    fun mint_internal(
        bars_account: &signer,
        artist: address,
        artist_name: vector<u8>,
        content_uri: vector<u8>,
        amount: u64
    ) {
        let token = NFT::create_for<BARSToken>(
            artist,
            create_bars_token(bars_account, artist_name),
            content_uri,
            amount,
            Option::none(),
        );
        NFTGallery::add_to_gallery(artist, token);
    }

    fun create_bars_token(address: &signer, artist_name: vector<u8>): BARSToken {
        assert(Std::Signer::address_of(address) == BARS, ENOT_BARS_OWNER);
        BARSToken { artist_name }
    }

    #[test_only]
    const EMINT_FAILED: u64 = 0;
    #[test_only]
    const ETRANSFER_FAILED: u64 = 1;
    #[test_only]
    const ArtistAddr: address = @0x42;

    #[test(admin=@DiemRoot, bars_account=@0x34CD440B72D03907100007D7BC0080A1, artist=@0x42, collector=@0x43)]
    public(script) fun test_bars(admin: signer, bars_account: signer, artist: signer, collector: signer) {
        NFT::nft_initialize(admin);

        register_user_internal(&artist);
        register_user_internal(&collector);

        let token_id = GUID::create_id(ArtistAddr, 0);
        mint_internal(&bars_account, Signer::address_of(&artist), b"kanye", b"yeezy.com", 7);

        assert(NFTGallery::has_token<BARSToken>(ArtistAddr, &token_id), EMINT_FAILED);
        assert(NFTGallery::get_token_balance<BARSToken>(ArtistAddr, &token_id) == 7, EMINT_FAILED);
        assert(NFT::supply<BARSToken>(&token_id) == 7, EMINT_FAILED);


        // Transfer 6 units of the token from creator to user
        NFTGallery::transfer_token_between_galleries<BARSToken>(
            artist, // from
            Signer::address_of(&collector), // to
            6, // amount
            ArtistAddr, // token.id.addr
            0, // token.id.creation_num
        );
        assert(NFTGallery::get_token_balance<BARSToken>(ArtistAddr, &token_id) == 1, ETRANSFER_FAILED);
        assert(NFTGallery::get_token_balance<BARSToken>(@0x43, &token_id) == 6, ETRANSFER_FAILED);
    }

}

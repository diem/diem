#[test_only]
module 0x1::NFTTests {
    use Std::GUID;
    use 0x1::NFT;
    use 0x1::NFTGallery;
    use Std::Option;

    /// A test token type to instantiate generic Tokens with.
    struct Game has store {
        name: vector<u8>,
        edition: u64,
    }

    struct Collection has store {
        name: vector<u8>,
    }

    struct Pokemon has store {
        name: vector<u8>,
        type: vector<u8>,
    }

    const EMINT_FAILED: u64 = 0;
    const ETRANSFER_FAILED: u64 = 1;
    const ECOLLECTION_FAILED: u64 = 2;

    #[test(admin=@0xa550c18, creator=@0x42, user=@0x43)]
    public(script) fun test_all(admin: signer, creator: signer, user: signer) {
        /*
        ===============================================================
            Initialization + preparation
        ===============================================================
        */

        let creator_addr = @0x42;
        let user_addr = @0x43;

        NFT::nft_initialize(admin);
        NFTGallery::publish_gallery<Game>(&creator);
        NFTGallery::publish_gallery<Collection>(&creator);
        NFTGallery::publish_gallery<Pokemon>(&creator);
        NFTGallery::publish_gallery<Game>(&user);

        let token1_id = GUID::create_id(creator_addr, 0);
        let token2_id = GUID::create_id(creator_addr, 1);

        /*
        ===============================================================
            Test minting
        ===============================================================
        */

        let token1 = NFT::create<Game>(
            &creator,
            Game { name: b"Mario", edition: 2008 },
            b"nintendo.com",
            10,
            Option::none(),
        );
        // Add all 10 tokens to creator's own account
        NFTGallery::add_to_gallery<Game>(creator_addr, token1);

        // Assert creator has the right number of tokens and supply is 10.
        assert(NFTGallery::has_token<Game>(creator_addr, &token1_id), EMINT_FAILED);
        assert(NFTGallery::get_token_balance<Game>(creator_addr, &token1_id) == 10, EMINT_FAILED);
        assert(NFT::supply<Game>(&token1_id) == 10, EMINT_FAILED);

        let token2 = NFT::create<Game>(
            &creator,
            Game { name: b"ChromeDino", edition: 2015 },
            b"google.com",
            233,
            Option::none(),
        );
        NFTGallery::add_to_gallery<Game>(creator_addr, token2);
        assert(NFTGallery::has_token<Game>(creator_addr, &token2_id), EMINT_FAILED);
        assert(NFTGallery::get_token_balance<Game>(creator_addr, &token2_id) == 233, EMINT_FAILED);

        /*
        ===============================================================
            Test collections
        ===============================================================
        */

        // Create collection first
        let collection = NFT::create<Collection>(
            &creator,
            Collection { name: b"Pokemon" },
            b"nintendo.com",
            1,
            Option::none(),
        );

        let pikachu = NFT::create<Pokemon>(
            &creator,
            Pokemon { name: b"Pikachu", type: b"electric", },
            b"nintendo.com",
            10,
            Option::some(NFT::id(&collection)),
        );
        let charmander = NFT::create<Pokemon>(
            &creator,
            Pokemon { name: b"Charmander", type: b"fire", },
            b"nintendo.com",
            10,
            Option::some(NFT::id(&collection)),
        );
        let pikachu_token = NFT::extract_token<Pokemon>(&pikachu);
        assert(NFT::parent(&pikachu_token) == &Option::some(NFT::id(&collection)), ECOLLECTION_FAILED);
        NFT::restore_token(pikachu_token);
        NFTGallery::add_to_gallery<Pokemon>(creator_addr, pikachu);

        let charmander_token = NFT::extract_token<Pokemon>(&charmander);
        assert(NFT::parent(&charmander_token) == &Option::some(NFT::id(&collection)), ECOLLECTION_FAILED);
        NFT::restore_token(charmander_token);
        NFTGallery::add_to_gallery<Pokemon>(creator_addr, charmander);
        NFTGallery::add_to_gallery<Collection>(creator_addr, collection);

        /*
        ===============================================================
            Test transferring tokens without splitting of tokens
        ===============================================================
        */

        // Transfer 6 units of token1 from creator to user
        NFTGallery::transfer_token_between_galleries<Game>(
            creator, // from
            user_addr, // to
            6, // amount
            creator_addr, // token.id.addr
            0, // token.id.creation_num
        );

        assert(NFTGallery::has_token<Game>(creator_addr, &token1_id), ETRANSFER_FAILED);
        assert(NFTGallery::get_token_balance<Game>(creator_addr, &token1_id) == 4, ETRANSFER_FAILED);
        assert(NFTGallery::has_token<Game>(user_addr, &token1_id), ETRANSFER_FAILED);
        assert(NFTGallery::get_token_balance<Game>(user_addr, &token1_id) == 6, ETRANSFER_FAILED);
        assert(NFT::supply<Game>(&token1_id) == 10, ETRANSFER_FAILED); // supply should not change


        /*
        ===============================================================
            Test transferring tokens with splitting of tokens
        ===============================================================
        */

        // Transfer all 6 units of token1 from user to creator
        NFTGallery::transfer_token_between_galleries<Game>(
            user, creator_addr, 6, creator_addr, 0,
        );
        assert(!NFTGallery::has_token<Game>(user_addr, &token1_id), ETRANSFER_FAILED); // user doesn't have token1 anymore
        assert(NFTGallery::get_token_balance<Game>(user_addr, &token1_id) == 0, ETRANSFER_FAILED);
        assert(NFTGallery::has_token<Game>(creator_addr, &token1_id), ETRANSFER_FAILED);
        assert(NFTGallery::get_token_balance<Game>(creator_addr, &token1_id) == 10, ETRANSFER_FAILED);
    }
}

module 0x1::NFTGallery {
    use Std::Errors;
    use Std::GUID;
    use 0x1::NFT::{Self, Token};
    use Std::Option::{Self, Option};
    use Std::Signer;
    use Std::Vector;

    /// Gallery holding tokens of `TokenType` as well as information of approved operators.
    struct NFTGallery<phantom TokenType: store> has key {
        gallery: vector<Token<TokenType>>
    }

    // Error codes
    const EID_NOT_FOUND: u64 = 0;
    const EBALANCE_NOT_PUBLISHED: u64 = 1;
    const EBALANCE_ALREADY_PUBLISHED: u64 = 2;
    const EINVALID_AMOUNT_OF_TRANSFER: u64 = 3;
    const EALREADY_IS_OPERATOR: u64 = 4;
    const ENOT_OPERATOR: u64 = 5;
    const EINVALID_APPROVAL_TARGET: u64 = 6;

    /// Add a token to the owner's gallery. If there is already a token of the same id in the
    /// gallery, we combine it with the new one and make a token of greater value.
    public fun add_to_gallery<TokenType: store>(owner: address, token: Token<TokenType>)
    acquires NFTGallery {
        assert!(exists<NFTGallery<TokenType>>(owner), EBALANCE_NOT_PUBLISHED);
        let id = NFT::id<TokenType>(&token);
        if (has_token<TokenType>(owner, &id)) {
            // If `owner` already has a token with the same id, remove it from the gallery
            // and join it with the new token.
            let original_token = remove_from_gallery<TokenType>(owner, &id);
            NFT::join<TokenType>(&mut token, original_token);
        };
        let gallery = &mut borrow_global_mut<NFTGallery<TokenType>>(owner).gallery;
        Vector::push_back(gallery, token)
    }

    /// Remove a token of certain id from the owner's gallery and return it.
    fun remove_from_gallery<TokenType: store>(owner: address, id: &GUID::ID): Token<TokenType>
    acquires NFTGallery {
        assert!(exists<NFTGallery<TokenType>>(owner), EBALANCE_NOT_PUBLISHED);
        let gallery = &mut borrow_global_mut<NFTGallery<TokenType>>(owner).gallery;
        let index_opt = index_of_token<TokenType>(gallery, id);
        assert!(Option::is_some(&index_opt), Errors::limit_exceeded(EID_NOT_FOUND));
        Vector::remove(gallery, Option::extract(&mut index_opt))
    }

    /// Finds the index of token with the given id in the gallery.
    fun index_of_token<TokenType: store>(gallery: &vector<Token<TokenType>>, id: &GUID::ID): Option<u64> {
        let i = 0;
        let len = Vector::length(gallery);
        while (i < len) {
            if (NFT::id<TokenType>(Vector::borrow(gallery, i)) == *id) {
                return Option::some(i)
            };
            i = i + 1;
        };
        Option::none()
    }

    /// Returns whether the owner has a token with given id.
    public fun has_token<TokenType: store>(owner: address, token_id: &GUID::ID): bool acquires NFTGallery {
        Option::is_some(&index_of_token(&borrow_global<NFTGallery<TokenType>>(owner).gallery, token_id))
    }

    public fun get_token_balance<TokenType: store>(owner: address, token_id: &GUID::ID
    ): u64 acquires NFTGallery {
        let gallery = &borrow_global<NFTGallery<TokenType>>(owner).gallery;
        let index_opt = index_of_token<TokenType>(gallery, token_id);

        if (Option::is_none(&index_opt)) {
            0
        } else {
            let index = Option::extract(&mut index_opt);
            NFT::balance(Vector::borrow(gallery, index))
        }
    }

    public fun transfer<TokenType: store>(account: &signer, to: address, amount: u64, token_id: &GUID::ID) acquires NFTGallery {
        let owner = Signer::address_of(account);

        assert!(amount > 0, EINVALID_AMOUNT_OF_TRANSFER);

        // Remove NFT from `owner`'s gallery
        let token = remove_from_gallery<TokenType>(owner, token_id);

        assert!(amount <= NFT::balance(&token), EINVALID_AMOUNT_OF_TRANSFER);

        if (amount == NFT::balance(&token)) {
            // Owner does not have any token left, so add token to `to`'s gallery.
            add_to_gallery<TokenType>(to, token);
        } else {
            // Split owner's token into two
            let (owner_token, to_token) = NFT::split<TokenType>(token, amount);

            // Add tokens to owner's gallery.
            add_to_gallery<TokenType>(owner, owner_token);

            // Add tokens to `to`'s gallery
            add_to_gallery<TokenType>(to, to_token);
        };

        // Emit transfer event
        NFT::emit_transfer_event(
            token_id,
            account,
            to,
            amount,
        );
    }

    /// Transfer `amount` of token with id `GUID::id(creator, creation_num)` from `owner`'s
    /// balance to `to`'s balance. This operation has to be done by either the owner or an
    /// approved operator of the owner.
    public(script) fun transfer_token_between_galleries<TokenType: store>(
        account: signer,
        to: address,
        amount: u64,
        creator: address,
        creation_num: u64
    ) acquires NFTGallery {
        transfer<TokenType>(&account, to, amount, &GUID::create_id(creator, creation_num));
    }

    public fun publish_gallery<TokenType: store>(account: &signer) {
        assert!(!exists<NFTGallery<TokenType>>(Signer::address_of(account)), EBALANCE_ALREADY_PUBLISHED);
        move_to(account, NFTGallery<TokenType> { gallery: Vector::empty() });
    }
}

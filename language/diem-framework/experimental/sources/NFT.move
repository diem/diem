address 0x1 {
module NFT {
    use Std::Errors;
    use Std::Event;
    use Std::GUID;
    use Std::Option::{Self, Option};
    use Std::Signer;
    use Std::Vector;

    /// Struct representing data of a specific token with token_id,
    /// stored under the creator's address inside TokenDataCollection.
    /// For each token_id, there is only one TokenData.
    struct TokenData<TokenType: store> has key, store {
        metadata: Option<TokenType>,
        /// Identifier for the token.
        token_id: GUID::GUID,
        /// Pointer to where the content and metadata is stored.
        content_uri: vector<u8>,
        supply: u64,
        /// Parent NFT id
        parent_id: Option<GUID::ID>
    }

    /// A hot potato wrapper for the token's metadata. Since this wrapper has no `key` or `store`
    /// ability, it can't be stored in global storage. This wrapper can be safely passed outside
    /// of this module because we know it will have to come back to this module, where
    /// it will be unpacked.
    struct TokenDataWrapper<TokenType: store> {
        origin: address,
        index: u64,
        metadata: TokenType,
        parent_id: Option<GUID::ID>,
    }

    /// Struct representing a semi-fungible or non-fungible token (depending on the supply).
    /// There can be multiple tokens with the same id (unless supply is 1). Each token's
    /// corresponding token metadata is stored inside a TokenData inside TokenDataCollection
    /// under the creator's address.
    struct Token<phantom TokenType: store> has key, store {
        id: GUID::ID,
        balance: u64,
    }

    struct MintEvent has copy, drop, store {
        id: GUID::ID,
        creator: address,
        content_uri: vector<u8>,
        amount: u64,
    }

    struct Admin has key {
        mint_events: Event::EventHandle<MintEvent>,
    }

    struct TokenDataCollection<TokenType: store> has key {
        tokens: vector<TokenData<TokenType>>,
    }

    /// Indicates that a user allows creation delegation for a given TokenType
    struct CreationDelegation<phantom TokenType: store> has key, store {
        guid_capability: GUID::CreateCapability,
    }

    const ADMIN: address = @0xa550c18;
    const MAX_U64: u64 = 18446744073709551615u64;
    // Error codes
    /// Function can only be called by the admin address
    const ENOT_ADMIN: u64  = 0;
    const EWRONG_TOKEN_ID: u64 = 1;
    const ETOKEN_BALANCE_OVERFLOWS: u64 = 2;
    const EAMOUNT_EXCEEDS_TOKEN_BALANCE: u64 = 3;
    const ETOKEN_EXTRACTED: u64 = 4;
    const EINDEX_EXCEEDS_LENGTH: u64 = 5;
    const ETOKEN_PRESENT: u64 = 6;
    const EPARENT_NOT_SAME_ACCOUNT: u64 = 7;
    const ETOKEN_DATA_COLLECTION_ALREADY_PUBLISHED: u64 = 8;
    /// Creation delegation for a given token type is not allowed.
    const ECREATION_DELEGATION_NOT_ALLOWED: u64 = 9;

    /// Returns the id of given token
    public fun id<TokenType: store>(token: &Token<TokenType>): GUID::ID {
        *&token.id
    }

    /// Returns the balance of given token
    public fun balance<TokenType: store>(token: &Token<TokenType>): u64 {
        token.balance
    }

    public fun metadata<TokenType: store>(wrapper: &TokenDataWrapper<TokenType>): &TokenType {
        &wrapper.metadata
    }

    /// Returns ID of collection associated with token
    public fun parent<TokenType: store>(wrapper: &TokenDataWrapper<TokenType>): &Option<GUID::ID> {
        &wrapper.parent_id
    }

    /// Returns the supply of tokens with `id` on the chain.
    public fun supply<TokenType: store>(id: &GUID::ID): u64 acquires TokenDataCollection {
        let owner_addr = GUID::id_creator_address(id);
        let tokens = &mut borrow_global_mut<TokenDataCollection<TokenType>>(owner_addr).tokens;
        let index_opt = index_of_token<TokenType>(tokens, id);
        assert(Option::is_some(&index_opt), Errors::invalid_argument(EWRONG_TOKEN_ID));
        let index = Option::extract(&mut index_opt);
        Vector::borrow(tokens, index).supply
    }

    /// Extract the Token data of the given token into a hot potato wrapper.
    public fun extract_token<TokenType: store>(nft: &Token<TokenType>): TokenDataWrapper<TokenType> acquires TokenDataCollection {
        let owner_addr = GUID::id_creator_address(&nft.id);
        let tokens = &mut borrow_global_mut<TokenDataCollection<TokenType>>(owner_addr).tokens;
        let index_opt = index_of_token<TokenType>(tokens, &nft.id);
        assert(Option::is_some(&index_opt), Errors::invalid_argument(EWRONG_TOKEN_ID));
        let index = Option::extract(&mut index_opt);
        let item_opt = &mut Vector::borrow_mut(tokens, index).metadata;
        assert(Option::is_some(item_opt), Errors::invalid_state(ETOKEN_EXTRACTED));
        let metadata = Option::extract(item_opt);
        let parent_opt = &mut Vector::borrow_mut(tokens, index).parent_id;
        TokenDataWrapper { origin: owner_addr, index, metadata, parent_id: *parent_opt }
    }

    /// Restore the token in the wrapper back into global storage under original address.
    public fun restore_token<TokenType: store>(wrapper: TokenDataWrapper<TokenType>) acquires TokenDataCollection {
        let TokenDataWrapper { origin, index, metadata, parent_id: _ } = wrapper;
        let tokens = &mut borrow_global_mut<TokenDataCollection<TokenType>>(origin).tokens;
        assert(Vector::length(tokens) > index, EINDEX_EXCEEDS_LENGTH);
        let item_opt = &mut Vector::borrow_mut(tokens, index).metadata;
        assert(Option::is_none(item_opt), ETOKEN_PRESENT);
        Option::fill(item_opt, metadata);
    }

    /// Finds the index of token with the given id in the gallery.
    fun index_of_token<TokenType: store>(gallery: &vector<TokenData<TokenType>>, id: &GUID::ID): Option<u64> {
        let i = 0;
        let len = Vector::length(gallery);
        while (i < len) {
            if (GUID::eq_id(&Vector::borrow(gallery, i).token_id, id)) {
                return Option::some(i)
            };
            i = i + 1;
        };
        Option::none()
    }

    /// Join two tokens and return a new token with the combined value of the two.
    public fun join<TokenType: store>(token: &mut Token<TokenType>, other: Token<TokenType>) {
        let Token { id, balance } = other;
        assert(*&token.id == id, EWRONG_TOKEN_ID);
        assert(MAX_U64 - token.balance >= balance, ETOKEN_BALANCE_OVERFLOWS);
        token.balance = token.balance + balance
    }

    /// Split the token into two tokens, one with balance `amount` and the other one with balance
    public fun split<TokenType: store>(token: Token<TokenType>, amount: u64): (Token<TokenType>, Token<TokenType>) {
        assert(token.balance >= amount, EAMOUNT_EXCEEDS_TOKEN_BALANCE);
        token.balance = token.balance - amount;
        let id = *&token.id;
        (token,
        Token {
            id,
            balance: amount
        } )
    }

    /// Initialize this module
    public(script) fun nft_initialize(account: signer) {
        assert(Signer::address_of(&account) == ADMIN, ENOT_ADMIN);
        move_to(&account, Admin {
            mint_events: Event::new_event_handle<MintEvent>(&account),
        })
    }

    /// Create an NFT on behalf of the given user, in case a user explicitly approved this delegation for the given
    /// NFT type.
    /// Only the entity, which can create an object of `TokenType`, will be able to call this function.
    public fun create_for<TokenType: store>(
        creator: address, metadata: TokenType, content_uri: vector<u8>, amount: u64, parent_id: Option<GUID::ID>
    ): Token<TokenType> acquires CreationDelegation, Admin, TokenDataCollection {
        assert (exists<CreationDelegation<TokenType>>(creator), ECREATION_DELEGATION_NOT_ALLOWED);
        let guid_creation_cap = &borrow_global<CreationDelegation<TokenType>>(creator).guid_capability;
        let guid = GUID::create_with_capability(creator, guid_creation_cap);
        create_impl<TokenType>(
            creator,
            guid,
            metadata,
            content_uri,
            amount,
            parent_id
        )
    }

    /// Create a` TokenData<TokenType>` that wraps `metadata` and with balance of `amount`
    public fun create<TokenType: store>(
        account: &signer, metadata: TokenType, content_uri: vector<u8>, amount: u64, parent_id: Option<GUID::ID>
    ): Token<TokenType> acquires Admin, TokenDataCollection {
        let guid = GUID::create(account);
        if (!exists<TokenDataCollection<TokenType>>(Signer::address_of(account))) {
            move_to(account, TokenDataCollection { tokens: Vector::empty<TokenData<TokenType>>() });
        };
        create_impl<TokenType>(
            Signer::address_of(account),
            guid,
            metadata,
            content_uri,
            amount,
            parent_id
        )
    }

    fun create_impl<TokenType: store>(
        addr: address,
        guid: GUID::GUID,
        metadata: TokenType,
        content_uri: vector<u8>,
        amount: u64,
        parent_id: Option<GUID::ID>
    ): Token<TokenType> acquires Admin, TokenDataCollection {
        // If there is a parent, ensure it has the same creator
        // TODO: Do we just say the owner has the ability instead?
        if (Option::is_some(&parent_id)) {
            let parent_id = Option::borrow(&mut parent_id);
            assert(GUID::creator_address(&guid) == GUID::id_creator_address(parent_id), EPARENT_NOT_SAME_ACCOUNT);
        };
        Event::emit_event(
            &mut borrow_global_mut<Admin>(ADMIN).mint_events,
            MintEvent {
                id: GUID::id(&guid),
                creator: addr,
                content_uri: copy content_uri,
                amount,
            }
        );
        let id = GUID::id(&guid);
        let token_data_collection = &mut borrow_global_mut<TokenDataCollection<TokenType>>(addr).tokens;
        Vector::push_back(
            token_data_collection,
            TokenData { metadata: Option::some(metadata), token_id: guid, content_uri, supply: amount, parent_id }
        );
        Token { id, balance: amount }
    }

    public fun publish_token_data_collection<TokenType: store>(account: &signer) {
        assert(
            !exists<TokenDataCollection<TokenType>>(Signer::address_of(account)),
            ETOKEN_DATA_COLLECTION_ALREADY_PUBLISHED
        );
        move_to(account, TokenDataCollection<TokenType> { tokens: Vector::empty() });
    }

    /// Allow creation delegation for a given TokenType (the entity, which can generate a metadata of a given TokenType
    /// is going to be allowed to create an NFT on behalf of the user).
    /// This is useful in case a user is using a 3rd party app, which can create NFTs on their behalf.
    public fun allow_creation_delegation<TokenType: store>(account: &signer) {
        if (!exists<CreationDelegation<TokenType>>(Signer::address_of(account))) {
            move_to(account, CreationDelegation<TokenType> { guid_capability: GUID::gen_create_capability(account) });
            // In order to support creation delegation, prepare the token data collection ahead of time.
            if (!exists<TokenDataCollection<TokenType>>(Signer::address_of(account))) {
                move_to(account, TokenDataCollection { tokens: Vector::empty<TokenData<TokenType>>() });
            };
        };
    }
}
}

module 0x1::NFT {
    use Std::Event;
    use Std::GUID::{Self, GUID};
    use Std::Signer;

    /// A non-fungible token of a specific `Type`, created by `id.addr`.
    /// Anyone can create a `NFT`. The access control policy for creating an `NFT<Type>` should be defined in the
    /// logic for creating `Type`. For example, if only Michelangelo should be able to  create `NFT<MikePainting>`,
    /// the `MikePainting` type should only be creatable by Michelangelo's address.
    struct NFT<Type: store + drop> has key, store {
        /// A struct to enable type-specific fields that will be different for each Token.
        /// For example, `NFT<Painting>` with
        /// `struct Painting { name: vector<u84, painter: vector<u8>, year: u64, ... }`,
        /// Or, `NFT<DigitalPirateInGameItem> { item_type: u8, item_power: u8, ... }`. Mutable.
        token: Type,
        /// A globally unique identifier, which includes the address of the NFT
        /// creator (who may or may not be the same as the content creator). Immutable.
        token_id: GUID,
        /// pointer to where the content and metadata is stored. Could be a DiemID domain, IPFS, Dropbox url, etc. Immutable.
        content_uri: vector<u8>,
        /// cryptographic hash of the NFT's contents (e.g., hash of the bytes corresponding to a video)
        content_hash: vector<u8>
    }

    struct MintEvent<phantom Type> has copy, drop, store {
        id: GUID::ID,
        creator: address,
        content_uri: vector<u8>,
    }

    struct TransferEvent<phantom Type> has copy, drop, store {
        from: address,
        to: address,
    }

    struct Admin<phantom Type> has key {
        mint_events: Event::EventHandle<MintEvent<Type>>,
    }

    const ADMIN: address = @0xa550c18;

    // Error codes
    const ENOT_ADMIN: u64 = 0;

    public fun initialize<Type: store + drop>(account: &signer) {
        assert!(Signer::address_of(account) == ADMIN, ENOT_ADMIN);
        move_to(account, Admin { mint_events: Event::new_event_handle<MintEvent<Type>>(account) })
    }

    /// Create a` NFT<Type>` that wraps `token`
    public fun create<Type: store + drop>(
        account: &signer, token: Type, content_uri: vector<u8>
    ): NFT<Type> acquires Admin {
        let creator = Signer::address_of(account);
        let token_id = GUID::create(account);
        Event::emit_event(
            &mut borrow_global_mut<Admin<Type>>(ADMIN).mint_events,
            MintEvent {
                id: GUID::id(&token_id),
                creator,
                content_uri: copy content_uri
            }
        );
        // TODO: take this as input
        let content_hash = x"";
        NFT { token, token_id, content_uri, content_hash }
    }

    /// Publish the non-fungible token `nft` under `account`.
    public fun publish<Type: store + drop>(account: &signer, nft: NFT<Type>) {
        move_to(account, nft)
    }

    /// Remove the `NFT<Type>` under `account`
    public fun remove<Type: store + drop>(account: &signer): NFT<Type> acquires NFT {
        move_from<NFT<Type>>(Signer::address_of(account))
    }

    /// Return the globally unique identifier of `nft`
    public fun id<Type: store + drop>(nft: &NFT<Type>): &GUID {
        &nft.token_id
    }

    /// Return the creator of this NFT
    public fun creator<Type: store + drop>(nft: &NFT<Type>): address {
        GUID::creator_address(id<Type>(nft))
    }

    /// View the underlying token of a NFT
    public fun token<Type: store + drop>(nft: &NFT<Type>): &Type {
        &nft.token
    }
}

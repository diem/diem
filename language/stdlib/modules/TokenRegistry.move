address 0x1 {

    module TokenRegistry {
        use 0x1::Errors;
        use 0x1::Signer;


        resource struct IdCounter {
            count: u64,
        }

        resource struct TokenMetadata<CoinType> {
            id: u64,
            transferable: bool,
            // possible other metadata fields can be added here
        }

        resource struct TokenRegistryWithMintCapability<CoinType> {
            maker_account: address,
        }
        // TODO:
        // Currently this TokenRegistry is a stand alone module, allows for users to register their token.
        // The TokenMetadata is published under the issuer account to allow for distributed stoarge.
        // A deignated mint capability is defined in this module.
        // Follow-up work requires changes to the Libra module. there are two possibilities:
        // 1) keep the register function entirely in this module. This requires a unique mint capability.
        // 2) move registration to the Libra module, in order to provide Libra::MintCapability


        /// A property expected of a `IdCounter` resource didn't hold
        const EID_COUNTER: u64 = 1;
        /// A property expected of a `TokenMetadata` resource didn't hold
        const ETOKEN_REG: u64 = 2;


        // TODO: move to the CoreAddresses module in the future
        public fun TOKEN_REGISTRY_COUNTER_ADDRESS(): address {
            0xA550C18
        }

        /// Initialization of the `TokenRegistry` module; initializes
        /// the counter of unique IDs
        public fun initialize(
            config_account: &signer,
        ) {
            assert(
                !exists<IdCounter>(Signer::address_of(config_account)),
                Errors::already_published(EID_COUNTER)
            );
            move_to(config_account, IdCounter {count: 0});
        }

        /// Returns a globally unique ID
        fun get_fresh_id(): u64 acquires IdCounter{
            let addr = TOKEN_REGISTRY_COUNTER_ADDRESS();
            assert(exists<IdCounter>(addr), Errors::not_published(EID_COUNTER));
            let id = borrow_global_mut<IdCounter>(addr);
            id.count = id.count + 1;
            id.count
        }

        /// Registers a new token, place its TokenMetadata on the signer
        /// account and returns a designated mint capability
        public fun register<CoinType>(maker_account: &signer, 
                                    _t: &CoinType,                                
                                    transferable: bool,
        ): TokenRegistryWithMintCapability<CoinType> acquires IdCounter {
            assert(!exists<TokenMetadata<CoinType>>(Signer::address_of(maker_account)), Errors::already_published(ETOKEN_REG));
            // increments unique counter under global registry address  
            let unique_id = get_fresh_id(); 
            move_to<TokenMetadata<CoinType>>(
                maker_account,  
                TokenMetadata { id: unique_id, transferable}
            ); 
            let address = Signer::address_of(maker_account);
            TokenRegistryWithMintCapability<CoinType>{maker_account: address}
        }

        /// Asserts that `CoinType` is a registered type at the given address
        public fun assert_is_registered_at<CoinType> (registered_at: address){
            assert(exists<TokenMetadata<CoinType>>(registered_at), Errors::not_published(ETOKEN_REG));
        }

        /// Returns `true` if the type `CoinType` is a transferable token.
        /// Returns `false` otherwise.
        public fun is_transferable<CoinType>(registered_at: address): bool acquires TokenMetadata{
            assert_is_registered_at<CoinType>(registered_at);
            let metadata = borrow_global<TokenMetadata<CoinType>>(registered_at);
            metadata.transferable
        }
        
        /// Returns the global uniqe ID of `CoinType`
        public fun get_id<CoinType>(registered_at: address): u64 acquires TokenMetadata{
            assert_is_registered_at<CoinType>(registered_at);
            let metadata = borrow_global<TokenMetadata<CoinType>>(registered_at);
            metadata.id
        }

    }
}

          
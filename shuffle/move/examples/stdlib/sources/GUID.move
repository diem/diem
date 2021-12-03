/// A module for generating globally unique identifiers
module Std::GUID {
    use Std::Signer;

    /// A generator for new GUIDs.
    struct Generator has key {
        /// A monotonically increasing counter
        counter: u64,
    }

    /// A globally unique identifier derived from the sender's address and a counter
    struct GUID has drop, store {
        id: ID
    }

    /// A non-privileged identifier that can be freely created by anyone. Useful for looking up GUID's.
    struct ID has copy, drop, store {
        /// If creation_num is `i`, this is the `i+1`th GUID created by `addr`
        creation_num: u64,
        /// Address that created the GUID
        addr: address
    }

    /// A capability to create a privileged identifier on behalf of the given address
    struct CreateCapability has key, store, drop {
        addr: address
    }

    /// GUID generator must be published ahead of first usage of `create_with_capability` function.
    const EGUID_GENERATOR_NOT_PUBLISHED: u64 = 0;

    /// Generates a capability to create the privileged GUID on behalf of the signer
    // (also makes sure that the Generator is published under the signer account)
    public fun gen_create_capability(account: &signer): CreateCapability {
        let addr = Signer::address_of(account);
        if (!exists<Generator>(addr)) {
            move_to(account, Generator { counter: 0 })
        };
        CreateCapability { addr }
    }

    /// Create a non-privileged id from `addr` and `creation_num`
    public fun create_id(addr: address, creation_num: u64): ID {
        ID { creation_num, addr }
    }

    public fun create_with_capability(addr: address, _cap: &CreateCapability): GUID acquires Generator {
        assert!(exists<Generator>(addr), EGUID_GENERATOR_NOT_PUBLISHED);
        create_impl(addr)
    }

    /// Create and return a new GUID. Creates a `Generator` under `account`
    /// if it does not already have one
    public fun create(account: &signer): GUID acquires Generator {
        let addr = Signer::address_of(account);
        if (!exists<Generator>(addr)) {
            move_to(account, Generator { counter: 0 })
        };
        create_impl(addr)
    }

    fun create_impl(addr: address): GUID acquires Generator {
        let generator = borrow_global_mut<Generator>(addr);
        let creation_num = generator.counter;
        generator.counter = creation_num + 1;
        GUID { id: ID { creation_num, addr } }
    }

    /// Publish a Generator resource under `account`
    public fun publish_generator(account: &signer) {
        move_to(account, Generator { counter: 0 })
    }

    /// Get the non-privileged ID associated with a GUID
    public fun id(guid: &GUID): ID {
        *&guid.id
    }

    /// Return the account address that created the GUID
    public fun creator_address(guid: &GUID): address {
        guid.id.addr
    }

    /// Return the account address that created the GUID::ID
    public fun id_creator_address(id: &ID): address {
        id.addr
    }

    /// Return the creation number associated with the GUID
    public fun creation_num(guid: &GUID): u64 {
        guid.id.creation_num
    }

    /// Return the creation number associated with the GUID::ID
    public fun id_creation_num(id: &ID): u64 {
        id.creation_num
    }

    /// Return true if the GUID's ID is `id`
    public fun eq_id(guid: &GUID, id: &ID): bool {
        &guid.id == id
    }

    /// Return the number of the next GUID to be created by `addr`
    public fun get_next_creation_num(addr: address): u64 acquires Generator {
        if (!exists<Generator>(addr)) {
            0
        } else {
            borrow_global<Generator>(addr).counter
        }
    }
}

module 0x1::Account {
    use Std::Errors;
    use Std::Signer;

    /// Resource representing an account.
    struct Account has key, store {
        authentication_key: vector<u8>,
        sequence_number: u64,
    }

    const MAX_U64: u128 = 18446744073709551615;

    const EACCOUNT: u64 = 0;
    const ESEQUENCE_NUMBER_TOO_BIG: u64 = 1;

    native fun create_signer(addr: address): signer;

    /// Publishes a new `Account` resource under `new_address`.
    /// A signer representing `new_address` is returned. This way, the caller of this function
    /// can publish additional resources under `new_address`.
    public fun create_account<T>(
        _creator: &signer,
        new_address: address,
        authentication_key: vector<u8>
    ) : signer {

        // there cannot be an Account resource under new_addr already.
        assert(!exists<Account>(new_address), Errors::already_published(EACCOUNT));

        let new_account = create_signer(new_address);
        move_to(
            &new_account,
            Account {
                authentication_key,
                sequence_number: 0
            }
        );

        new_account
    }

    public fun get_sequence_number(addr: address) : u64 acquires Account {
        borrow_global<Account>(addr).sequence_number
    }

    public fun get_auth_key(addr: address) : vector<u8> acquires Account {
        *&borrow_global<Account>(addr).authentication_key
    }

    public fun rotate_authentication_key<T>(
        account: &signer,
        new_auth_key: vector<u8>,
    ) acquires Account {
        let account_resource = borrow_global_mut<Account>(Signer::address_of(account));
        account_resource.authentication_key = new_auth_key;
    }

    /// Epilogue function is run after a transaction is successfully executed.
    fun epilogue(account: &signer) acquires Account {
        let addr = Signer::address_of(account);
        let old_sequence_number = get_sequence_number(addr);

        assert(
            (old_sequence_number as u128) < MAX_U64,
            Errors::limit_exceeded(ESEQUENCE_NUMBER_TOO_BIG)
        );

        // Increment sequence number
        let account_resource = borrow_global_mut<Account>(addr);
        account_resource.sequence_number = old_sequence_number + 1;
    }
}

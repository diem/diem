address 0x0:

module ValidatorConfig2 {
    use 0x0::LibraAccount;
    use 0x0::Transaction;

    struct Config {
        consensus_pubkey: vector<u8>,
    }

    // A current or prospective validator should publish one of these under their accounts.
    // If delegation is enabled, only the delegated_account can change the config.
    // If delegation is disabled, only the owner of this resource can change the config.
    // The entity that can change the config is called a "validator operator".
    // The owner of this resource can enable delegation and set delegated account.
    // The owner can also disable delegation at any time.
    resource struct T {
        config: Config,
        delegation_enabled: bool,
        delegated_account: address,
    }

    // TODO(valerini): add events here

    // Returns true if addr has a published ValidatorConfig::T resource
    public fun has(addr: address): bool {
        exists<T>(addr)
    }

    // Get Config
    public fun get_config(addr: address): Config acquires T {
        *&borrow_global<T>(addr).config
    }

    // Get consensus_pubkey from Config
    public fun get_consensus_pubkey(config_ref: &Config): vector<u8> {
        *&config_ref.consensus_pubkey
    }

    // Returns the address of the entity who is now in charge of managing the config.
    public fun get_validator_operator_account(addr: address): address acquires T {
        let t_ref = borrow_global<T>(addr);
        if (t_ref.delegation_enabled) {
            return *&t_ref.delegated_account
        } else {
            return addr
        }
    }

    // Register the transaction sender as a candidate validator by creating a ValidatorConfig
    // resource under their account. Note that only one such resource can be instantiated under an account.
    public fun initialize(
        consensus_pubkey: vector<u8>,) {

        move_to_sender<T>(
            T {
                config: Config {
                    consensus_pubkey: consensus_pubkey,
                },
                delegation_enabled: false,
                delegated_account: 0x0
            }
        );
    }

    // If the sender of the transaction has a ValidatorConfig::T resource,
    // this function will delegate management of the ValidatorConfig::T resource to a delegated_account.
    public fun set_delegated_account(delegated_account: address) acquires T {
        Transaction::assert(LibraAccount::exists(delegated_account), 5);
        // check delegated address is different from transaction's sender
        Transaction::assert(delegated_account != Transaction::sender(), 6);
        let t_ref = borrow_global_mut<T>(Transaction::sender());
        *(&mut t_ref.delegation_enabled) = true;
        *(&mut t_ref.delegated_account) = delegated_account;
    }

    // If the sender of the transaction has a ValidatorConfig::T resource,
    // this function will revoke management capabilities from the delegated_account account.
    public fun remove_delegated_account() acquires T {
        let t_ref = borrow_global_mut<T>(Transaction::sender());
        *(&mut t_ref.delegation_enabled) = false;
        *(&mut t_ref.delegated_account) = 0x0;
    }

    // Rotate validator's config.
    // Here validator_account - is the account of the validator whose consensus_pubkey is going to be rotated.
    public fun rotate_consensus_pubkey(
        validator_account: address,
        new_consensus_pubkey: vector<u8>,
        _proof: vector<u8>
    ) acquires T {
        let addr = get_validator_operator_account(validator_account);
        Transaction::assert(Transaction::sender() == addr, 1);

        // TODO(valerini): verify the proof of posession of new_consensus_secretkey

        let t_ref = borrow_global_mut<T>(validator_account);
        // Assert that the new key is different from the old key
        Transaction::assert(&t_ref.config.consensus_pubkey != &new_consensus_pubkey, 15);
        // Set the new key
        let key_ref = &mut t_ref.config.consensus_pubkey;
        *key_ref = new_consensus_pubkey;
    }

    // Simplified arguments when the sender is the validators itself
    public fun rotate_consensus_pubkey_of_sender(new_consensus_pubkey: vector<u8>, proof: vector<u8>) acquires T {
        rotate_consensus_pubkey(Transaction::sender(), new_consensus_pubkey, proof);
    }
}

/// Maintains the version number for the blockchain.
module DiemFramework::CoreVersion {
    use Std::Capability::Cap;
    use Std::Errors;
    use Std::Signer;

    struct CoreVersion<phantom T> has key, copy, drop, store {
        major: u64,
    }

    /// Error with config
    const ECONFIG: u64 = 0;
    /// Tried to set an invalid major version for the VM. Major versions must be strictly increasing
    const EINVALID_MAJOR_VERSION_NUMBER: u64 = 1;

    /// Publishes the Version config.
    public fun initialize<T>(account: &signer, initial_version: u64) {
        assert(
            !exists<CoreVersion<T>>(Signer::address_of(account)),
            Errors::already_published(ECONFIG)
        );
        move_to(
            account,
            CoreVersion<T> { major: initial_version },
        );
    }

    spec initialize {
        let config_addr = Signer::address_of(account);
        aborts_if exists<CoreVersion<T>>(config_addr) with Errors::ALREADY_PUBLISHED;
        ensures exists<CoreVersion<T>>(config_addr);
        ensures global<CoreVersion<T>>(config_addr).major == initial_version;
    }

    /// Updates the major version to a larger version.
    public fun set<T>(config_addr: address, _cap: &Cap<T>, major: u64) acquires CoreVersion {
        assert(exists<CoreVersion<T>>(config_addr), Errors::not_published(ECONFIG));
        let old_major = *&borrow_global<CoreVersion<T>>(config_addr).major;

        assert(
            old_major < major,
            Errors::invalid_argument(EINVALID_MAJOR_VERSION_NUMBER)
        );

        let config = borrow_global_mut<CoreVersion<T>>(config_addr);
        config.major = major;
    }

    spec set {
        aborts_if !exists<CoreVersion<T>>(config_addr) with Errors::NOT_PUBLISHED;
        aborts_if global<CoreVersion<T>>(config_addr).major >= major with Errors::INVALID_ARGUMENT;
        ensures global<CoreVersion<T>>(config_addr).major == major;
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    spec module {
        /// Version number never decreases
        invariant<T> update [suspendable]
            forall config_addr: address:
                (old(global<CoreVersion<T>>(config_addr).major) <= global<CoreVersion<T>>(config_addr).major);
    }

}

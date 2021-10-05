/// Maintains the version number for the blockchain.
module DiemFramework::CoreVersion {
    use Std::Capability::Cap;
    use Std::Errors;
    use Std::Signer;

    /// Marker to be stored under 0x1 during genesis
    struct VersionChainMarker<phantom T> has key {}

    struct CoreVersion has key {
        major: u64,
    }

    /// Error with chain marker
    const ECHAIN_MARKER: u64 = 0;
    /// Error with config
    const ECONFIG: u64 = 1;
    /// Tried to set an invalid major version for the VM. Major versions must be strictly increasing
    const EINVALID_MAJOR_VERSION_NUMBER: u64 = 2;

    /// Publishes the Version config.
    public fun initialize<T>(account: &signer, initial_version: u64) {
        assert!(Signer::address_of(account) == @ConfigStorage, Errors::requires_address(ECONFIG));

        assert!(
            !exists<VersionChainMarker<T>>(@ConfigStorage),
            Errors::already_published(ECHAIN_MARKER)
        );

        assert!(
            !exists<CoreVersion>(@ConfigStorage),
            Errors::already_published(ECONFIG)
        );
        move_to(
            account,
            VersionChainMarker<T> {},
        );
        move_to(
            account,
            CoreVersion { major: initial_version },
        );
    }

    spec initialize {
        aborts_if exists<CoreVersion>(@ConfigStorage) with Errors::ALREADY_PUBLISHED;
        ensures exists<CoreVersion>(@ConfigStorage);
        ensures global<CoreVersion>(@ConfigStorage).major == initial_version;
    }

    /// Updates the major version to a larger version.
    public fun set<T>(major: u64, _cap: &Cap<T>) acquires CoreVersion {
        assert!(exists<VersionChainMarker<T>>(@ConfigStorage), Errors::not_published(ECHAIN_MARKER));
        assert!(exists<CoreVersion>(@ConfigStorage), Errors::not_published(ECONFIG));
        let old_major = *&borrow_global<CoreVersion>(@ConfigStorage).major;

        assert!(
            old_major < major,
            Errors::invalid_argument(EINVALID_MAJOR_VERSION_NUMBER)
        );

        let config = borrow_global_mut<CoreVersion>(@ConfigStorage);
        config.major = major;
    }

    spec set {
        aborts_if !exists<VersionChainMarker<T>>(@ConfigStorage) with Errors::NOT_PUBLISHED;
        aborts_if !exists<CoreVersion>(@ConfigStorage) with Errors::NOT_PUBLISHED;
        aborts_if global<CoreVersion>(@ConfigStorage).major >= major with Errors::INVALID_ARGUMENT;
        ensures global<CoreVersion>(@ConfigStorage).major == major;
    }

}

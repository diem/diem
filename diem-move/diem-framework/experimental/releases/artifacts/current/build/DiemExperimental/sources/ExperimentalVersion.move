/// Maintains the version number for the blockchain.
module ExperimentalFramework::ExperimentalVersion {
    use CoreFramework::DiemVersion;
    // TODO: remove dependency on DPN framework
    use DiemFramework::DiemConfig;
    use Std::Capability;
    use Std::Signer;
    use ExperimentalFramework::ExperimentalChainMarker::{ExperimentalChainMarker, get_chain_marker};

//    struct ExperimentalVersion has drop {}

    /// Publishes the Version config.
    public fun initialize(account: &signer, initial_version: u64) {
//        assert!(Signer::address_of(account) == @0x42, 0);
        DiemVersion::initialize<ExperimentalChainMarker>(account, initial_version);
        Capability::create<ExperimentalChainMarker>(account, &get_chain_marker());
    }

    /// Updates the major version to a larger version.
    public fun set(account: &signer, major: u64) {
        let config_addr = Signer::address_of(account);
        DiemVersion::set<ExperimentalChainMarker>(
            config_addr,
            &Capability::acquire(account, &get_chain_marker()),
            major
        );
        DiemConfig::reconfigure(account);
    }

//    spec fun set {
//        aborts_if !can_acquire_cap<ExperimentalMarker>(addr);
//    }
//
//    invariant forall addr: address where can_acquire_cap<ExperimentalMarker>(addr): addr == @0x42;
}

/// Maintains the version number for the blockchain.
module DiemFramework::ExperimentalVersion {
    use DiemFramework::CoreVersion;
    use DiemFramework::DiemConfig;
    use Std::Capability;
    use Std::Signer;

    struct ExperimentalVersion has drop {}

    /// Publishes the Version config.
    public fun initialize(account: &signer, initial_version: u64) {
        CoreVersion::initialize<ExperimentalVersion>(account, initial_version);
        Capability::create(account, &ExperimentalVersion{});
    }

    /// Updates the major version to a larger version.
    public fun set(account: &signer, major: u64) {
        let config_addr = Signer::address_of(account);
        CoreVersion::set<ExperimentalVersion>(
            config_addr,
            &Capability::acquire(account, &ExperimentalVersion{}),
            major
        );
        DiemConfig::reconfigure(account);
    }
}

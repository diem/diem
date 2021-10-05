/// Maintains the version number for the blockchain.
module DiemFramework::ExperimentalVersion {
    use DiemFramework::CoreVersion;
    use DiemFramework::DiemConfig;
    use Std::Capability;

    struct ExperimentalVersion has drop {}

    /// Publishes the Version config.
    public fun initialize(account: &signer, initial_version: u64) {
        CoreVersion::initialize<ExperimentalVersion>(account, initial_version);
        Capability::create(account, &ExperimentalVersion{});
    }

    /// Updates the major version to a larger version.
    public fun set(account: &signer, major: u64) {
        CoreVersion::set<ExperimentalVersion>(
            major,
            &Capability::acquire(account, &ExperimentalVersion{})
        );
        DiemConfig::reconfigure(account);
    }
}

address 0x1 {

module LibraVersion {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::LibraConfig;
    use 0x1::LibraTimestamp;

    struct LibraVersion {
        major: u64,
    }

    spec module {
    }

    const EINVALID_MAJOR_VERSION_NUMBER: u64 = 0;

    public fun initialize(
        lr_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        CoreAddresses::assert_libra_root(lr_account);
        LibraConfig::publish_new_config<LibraVersion>(
            lr_account,
            LibraVersion { major: 1 },
        );
    }

    public fun set(account: &signer, major: u64) {
        LibraTimestamp::assert_operating();

        // TODO: is this restricted to be called from libra root?
        // CoreAddresses::assert_libra_root(account);

        let old_config = LibraConfig::get<LibraVersion>();

        assert(
            old_config.major < major,
            Errors::invalid_argument(EINVALID_MAJOR_VERSION_NUMBER)
        );

        LibraConfig::set<LibraVersion>(
            account,
            LibraVersion { major }
        );
    }

    spec module {
        /// After genesis, version is published.
        invariant [global] LibraTimestamp::is_operating() ==> LibraConfig::spec_is_published<LibraVersion>();

        /// The permission "UpdateLibraProtocolVersion" is granted to LibraRoot [B20].
        invariant [global, on_update] forall addr: address where exists<LibraVersion>(addr):
            addr == CoreAddresses::LIBRA_ROOT_ADDRESS();
    }
}

}

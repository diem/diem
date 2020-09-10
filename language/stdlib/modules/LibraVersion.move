address 0x1 {

module LibraVersion {
    use 0x1::CoreAddresses;
    use 0x1::Errors;
    use 0x1::LibraConfig::{Self, LibraConfig};
    use 0x1::LibraTimestamp;
    use 0x1::Roles;

    struct LibraVersion {
        major: u64,
    }

    /// Tried to set an invalid major version for the VM. Major versions must be strictly increasing
    const EINVALID_MAJOR_VERSION_NUMBER: u64 = 0;


    spec module {
        /// Verify all functions in this module.
        pragma verify;
    }

    public fun initialize(
        lr_account: &signer,
    ) {
        LibraTimestamp::assert_genesis();
        Roles::assert_libra_root(lr_account);
        LibraConfig::publish_new_config<LibraVersion>(
            lr_account,
            LibraVersion { major: 1 },
        );
    }
    spec fun initialize {
        /// Must abort if the signer does not have the LibraRoot role [B19].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};

        include LibraTimestamp::AbortsIfNotGenesis;
        include LibraConfig::PublishNewConfigAbortsIf<LibraVersion>;
        include LibraConfig::PublishNewConfigEnsures<LibraVersion>{payload: LibraVersion { major: 1 }};
    }

    public fun set(lr_account: &signer, major: u64) {
        LibraTimestamp::assert_operating();

        Roles::assert_libra_root(lr_account);

        let old_config = LibraConfig::get<LibraVersion>();

        assert(
            old_config.major < major,
            Errors::invalid_argument(EINVALID_MAJOR_VERSION_NUMBER)
        );

        LibraConfig::set<LibraVersion>(
            lr_account,
            LibraVersion { major }
        );
    }
    spec fun set {
        /// Must abort if the signer does not have the LibraRoot role [B19].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};

        include LibraTimestamp::AbortsIfNotOperating;
        aborts_if LibraConfig::get<LibraVersion>().major >= major with Errors::INVALID_ARGUMENT;
        include LibraConfig::SetAbortsIf<LibraVersion>{account: lr_account};
        include LibraConfig::SetEnsures<LibraVersion>{payload: LibraVersion { major }};
    }

    spec module {
        /// After genesis, version is published.
        invariant [global] LibraTimestamp::is_operating() ==> LibraConfig::spec_is_published<LibraVersion>();

        /// The permission "UpdateLibraProtocolVersion" is granted to LibraRoot [B19].
        invariant [global, isolated] forall addr: address where exists<LibraConfig<LibraVersion>>(addr):
            addr == CoreAddresses::LIBRA_ROOT_ADDRESS();
    }

    /// Only "set" can modify the LibraVersion config [B19]
    spec schema LibraVersionRemainsSame {
        ensures old(LibraConfig::spec_is_published<LibraVersion>()) ==>
            global<LibraConfig<LibraVersion>>(CoreAddresses::LIBRA_ROOT_ADDRESS()) ==
                old(global<LibraConfig<LibraVersion>>(CoreAddresses::LIBRA_ROOT_ADDRESS()));
    }
    spec module {
        apply LibraVersionRemainsSame to * except set;
    }
}
}

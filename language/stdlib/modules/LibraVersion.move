address 0x1 {


/// Maintains the version number for the Libra blockchain. The version is stored in a
/// LibraConfig, and may be updated by Libra root.
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

    /// Publishes the LibraVersion config. Must be called during Genesis.
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
        /// Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};

        include LibraTimestamp::AbortsIfNotGenesis;
        include LibraConfig::PublishNewConfigAbortsIf<LibraVersion>;
        include LibraConfig::PublishNewConfigEnsures<LibraVersion>{payload: LibraVersion { major: 1 }};
    }

    /// Allows Libra root to update the major version to a larger version.
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
        /// Must abort if the signer does not have the LibraRoot role [[H10]][PERMISSION].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};

        include LibraTimestamp::AbortsIfNotOperating;
        aborts_if LibraConfig::get<LibraVersion>().major >= major with Errors::INVALID_ARGUMENT;
        include LibraConfig::SetAbortsIf<LibraVersion>{account: lr_account};
        include LibraConfig::SetEnsures<LibraVersion>{payload: LibraVersion { major }};
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Initialization
    spec module {
        /// After genesis, version is published.
        invariant [global] LibraTimestamp::is_operating() ==> LibraConfig::spec_is_published<LibraVersion>();
    }

    /// # Access Control

    /// Only "set" can modify the LibraVersion config [[H10]][PERMISSION]
    spec schema LibraVersionRemainsSame {
        ensures old(LibraConfig::spec_is_published<LibraVersion>()) ==>
            global<LibraConfig<LibraVersion>>(CoreAddresses::LIBRA_ROOT_ADDRESS()) ==
                old(global<LibraConfig<LibraVersion>>(CoreAddresses::LIBRA_ROOT_ADDRESS()));
    }
    spec module {
        apply LibraVersionRemainsSame to * except set;
    }

    spec module {
        /// The permission "UpdateLibraProtocolVersion" is granted to LibraRoot [[H10]][PERMISSION].
        invariant [global, isolated] forall addr: address where exists<LibraConfig<LibraVersion>>(addr):
            addr == CoreAddresses::LIBRA_ROOT_ADDRESS();
    }

    /// # Other Invariants
    spec module {
        /// Version number never decreases
        invariant update [global, isolated]
            old(LibraConfig::get<LibraVersion>().major) <= LibraConfig::get<LibraVersion>().major;
    }

}
}

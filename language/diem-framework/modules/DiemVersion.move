/// Maintains the version number for the Diem blockchain. The version is stored in a
/// DiemConfig, and may be updated by Diem root.
module DiemFramework::DiemVersion {
    use DiemFramework::DiemConfig::{Self, DiemConfig};
    use DiemFramework::DiemTimestamp;
    use DiemFramework::Roles;
    use Std::Errors;

    struct DiemVersion has copy, drop, store {
        major: u64,
    }

    /// Tried to set an invalid major version for the VM. Major versions must be strictly increasing
    const EINVALID_MAJOR_VERSION_NUMBER: u64 = 0;

    /// Publishes the DiemVersion config. Must be called during Genesis.
    public fun initialize(dr_account: &signer, initial_version: u64) {
        DiemTimestamp::assert_genesis();
        Roles::assert_diem_root(dr_account);
        DiemConfig::publish_new_config<DiemVersion>(
            dr_account,
            DiemVersion { major: initial_version },
        );
    }
    spec initialize {
        /// Must abort if the signer does not have the DiemRoot role [[H10]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemTimestamp::AbortsIfNotGenesis;
        include DiemConfig::PublishNewConfigAbortsIf<DiemVersion>;
        include DiemConfig::PublishNewConfigEnsures<DiemVersion>{payload: DiemVersion { major: initial_version }};
    }

    /// Allows Diem root to update the major version to a larger version.
    public fun set(dr_account: &signer, major: u64) {
        DiemTimestamp::assert_operating();

        Roles::assert_diem_root(dr_account);

        let old_config = DiemConfig::get<DiemVersion>();

        assert(
            old_config.major < major,
            Errors::invalid_argument(EINVALID_MAJOR_VERSION_NUMBER)
        );

        DiemConfig::set<DiemVersion>(
            dr_account,
            DiemVersion { major }
        );
    }
    spec set {
        /// Must abort if the signer does not have the DiemRoot role [[H10]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemTimestamp::AbortsIfNotOperating;
        aborts_if DiemConfig::get<DiemVersion>().major >= major with Errors::INVALID_ARGUMENT;
        include DiemConfig::SetAbortsIf<DiemVersion>{account: dr_account};
        include DiemConfig::SetEnsures<DiemVersion>{payload: DiemVersion { major }};
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Initialization
    spec module {
        /// After genesis, version is published.
        invariant [suspendable] DiemTimestamp::is_operating() ==> DiemConfig::spec_is_published<DiemVersion>();
    }

    /// # Access Control

    /// The permission "UpdateDiemProtocolVersion" is granted to DiemRoot [[H10]][PERMISSION].
    spec module {
        invariant forall addr: address
            where exists<DiemConfig<DiemVersion>>(addr): addr == @DiemRoot;

        invariant update [suspendable] old(DiemConfig::spec_is_published<DiemVersion>())
            && DiemConfig::spec_is_published<DiemVersion>()
            && old(DiemConfig::get<DiemVersion>().major) != DiemConfig::get<DiemVersion>().major
                ==> Roles::spec_signed_by_diem_root_role();
    }

    // TODO: The following is the old style spec, which can removed later.
    /// Only "set" can modify the DiemVersion config [[H10]][PERMISSION]
    spec schema DiemVersionRemainsSame {
        ensures old(DiemConfig::spec_is_published<DiemVersion>()) ==>
            global<DiemConfig<DiemVersion>>(@DiemRoot) ==
                old(global<DiemConfig<DiemVersion>>(@DiemRoot));
    }
    /// The permission "UpdateDiemProtocolVersion" is granted to DiemRoot [[H10]][PERMISSION].
    spec module {
        apply DiemVersionRemainsSame to * except set;
    }

    /// # Other Invariants
    spec module {
        /// Version number never decreases
        invariant update [suspendable]
            DiemTimestamp::is_operating() ==>
                (old(DiemConfig::get<DiemVersion>().major) <= DiemConfig::get<DiemVersion>().major);
    }

}

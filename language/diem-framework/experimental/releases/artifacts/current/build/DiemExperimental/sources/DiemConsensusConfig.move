/// Maintains the consensus config for the Diem blockchain. The config is stored in a
/// DiemConfig, and may be updated by Diem root.
module DiemFramework::DiemConsensusConfig {
    use DiemFramework::DiemConfig::{Self, DiemConfig};
    use DiemFramework::Roles;
    use Std::Vector;

    struct DiemConsensusConfig has copy, drop, store {
        config: vector<u8>,
    }

    /// Publishes the DiemConsensusConfig config.
    public fun initialize(dr_account: &signer) {
        Roles::assert_diem_root(dr_account);
        DiemConfig::publish_new_config(dr_account, DiemConsensusConfig { config: Vector::empty() });
    }
    spec initialize {
        /// Must abort if the signer does not have the DiemRoot role [[H12]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemConfig::PublishNewConfigAbortsIf<DiemConsensusConfig>;
        include DiemConfig::PublishNewConfigEnsures<DiemConsensusConfig>{
            payload: DiemConsensusConfig { config: Vector::empty() }
        };
    }

    /// Allows Diem root to update the config.
    public fun set(dr_account: &signer, config: vector<u8>) {
        Roles::assert_diem_root(dr_account);

        DiemConfig::set(
            dr_account,
            DiemConsensusConfig { config }
        );
    }
    spec set {
        /// Must abort if the signer does not have the DiemRoot role [[H12]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemConfig::SetAbortsIf<DiemConsensusConfig>{account: dr_account};
        include DiemConfig::SetEnsures<DiemConsensusConfig>{payload: DiemConsensusConfig { config }};
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Access Control

    /// The permission "UpdateDiemConsensusConfig" is granted to DiemRoot [[H12]][PERMISSION].
    spec module {
        invariant [suspendable] forall addr: address
            where exists<DiemConfig<DiemConsensusConfig>>(addr): addr == @DiemRoot;

        invariant update [suspendable] old(DiemConfig::spec_is_published<DiemConsensusConfig>())
            && DiemConfig::spec_is_published<DiemConsensusConfig>()
            && old(DiemConfig::get<DiemConsensusConfig>()) != DiemConfig::get<DiemConsensusConfig>()
                ==> Roles::spec_signed_by_diem_root_role();
    }

    // TODO: The following is the old style spec, which can removed later.
    /// Only "set" can modify the DiemConsensusConfig config [[H12]][PERMISSION]
    spec schema DiemConsensusConfigRemainsSame {
        ensures old(DiemConfig::spec_is_published<DiemConsensusConfig>()) ==>
            global<DiemConfig<DiemConsensusConfig>>(@DiemRoot) ==
                old(global<DiemConfig<DiemConsensusConfig>>(@DiemRoot));
    }
    spec module {
        apply DiemConsensusConfigRemainsSame to * except set;
    }
}

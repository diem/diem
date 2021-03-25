address 0x1 {
/// Maintains the consensus config for the Diem blockchain. The config is stored in a
/// DiemConfig, and may be updated by Diem root.
module DiemConsensusConfig {
    use 0x1::CoreAddresses;
    use 0x1::DiemConfig::{Self, DiemConfig};
    use 0x1::DiemTimestamp;
    use 0x1::Roles;
    use 0x1::Vector;

    struct DiemConsensusConfig has copy, drop, store {
        config: vector<u8>,
    }

    /// Publishes the DiemConsensusConfig config.
    public fun initialize(dr_account: &signer) {
        Roles::assert_diem_root(dr_account);
        DiemConfig::publish_new_config(dr_account, DiemConsensusConfig { config: Vector::empty() });
    }
    spec fun initialize {
        /// Must abort if the signer does not have the DiemRoot role [[H12]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemConfig::PublishNewConfigAbortsIf<DiemConsensusConfig>;
        include DiemConfig::PublishNewConfigEnsures<DiemConsensusConfig>{
            payload: DiemConsensusConfig { config: Vector::empty() }
        };
    }

    /// Allows Diem root to update the config.
    public fun set(dr_account: &signer, config: vector<u8>) {
        DiemTimestamp::assert_operating();

        Roles::assert_diem_root(dr_account);

        DiemConfig::set(
            dr_account,
            DiemConsensusConfig { config }
        );
    }
    spec fun set {
        /// Must abort if the signer does not have the DiemRoot role [[H12]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemTimestamp::AbortsIfNotOperating;
        include DiemConfig::SetAbortsIf<DiemConsensusConfig>{account: dr_account};
        include DiemConfig::SetEnsures<DiemConsensusConfig>{payload: DiemConsensusConfig { config }};
    }

    // =================================================================
    // Module Specification

    spec module {} // Switch to module documentation context

    /// # Access Control

    /// Only "set" can modify the DiemConsensusConfig config [[H12]][PERMISSION]
    spec schema DiemConsensusConfigRemainsSame {
        ensures old(DiemConfig::spec_is_published<DiemConsensusConfig>()) ==>
            global<DiemConfig<DiemConsensusConfig>>(CoreAddresses::DIEM_ROOT_ADDRESS()) ==
                old(global<DiemConfig<DiemConsensusConfig>>(CoreAddresses::DIEM_ROOT_ADDRESS()));
    }
    spec module {
        apply DiemConsensusConfigRemainsSame to * except set;
    }

    spec module {
        /// The permission "UpdateDiemConsensusConfig" is granted to DiemRoot [[H12]][PERMISSION].
        invariant [global, isolated] forall addr: address where exists<DiemConfig<DiemConsensusConfig>>(addr):
            addr == CoreAddresses::DIEM_ROOT_ADDRESS();
    }
}
}

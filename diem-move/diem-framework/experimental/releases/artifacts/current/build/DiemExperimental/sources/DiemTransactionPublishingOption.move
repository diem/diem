/// This module defines a struct storing the publishing policies for the VM.
module DiemFramework::DiemTransactionPublishingOption {
    use DiemFramework::DiemConfig::{Self, DiemConfig};
    use DiemFramework::DiemTimestamp;
    use DiemFramework::Roles;
    use Std::Errors;
    use Std::Signer;
    use Std::Vector;

    const SCRIPT_HASH_LENGTH: u64 = 32;

    /// The script hash has an invalid length
    const EINVALID_SCRIPT_HASH: u64 = 0;
    /// The script hash already exists in the allowlist
    const EALLOWLIST_ALREADY_CONTAINS_SCRIPT: u64 = 1;
    /// Attempting to publish/unpublish a HaltAllTransactions resource that does not exist.
    const EHALT_ALL_TRANSACTIONS: u64 = 2;

    /// Defines and holds the publishing policies for the VM. There are three possible configurations:
    /// 1. No module publishing, only allow-listed scripts are allowed.
    /// 2. No module publishing, custom scripts are allowed.
    /// 3. Both module publishing and custom scripts are allowed.
    /// We represent these as the following resource.
    struct DiemTransactionPublishingOption has copy, drop, store {
        /// Only script hashes in the following list can be executed by the network. If the vector is empty, no
        /// limitation would be enforced.
        script_allow_list: vector<vector<u8>>,
        /// Anyone can publish new module if this flag is set to true.
        module_publishing_allowed: bool,
    }

    /// If published, halts transactions from all accounts except DiemRoot
    struct HaltAllTransactions has key {}

    public fun initialize(
        dr_account: &signer,
        script_allow_list: vector<vector<u8>>,
        module_publishing_allowed: bool,
    ) {
        DiemTimestamp::assert_genesis();
        Roles::assert_diem_root(dr_account);

        DiemConfig::publish_new_config(
            dr_account,
            DiemTransactionPublishingOption {
                script_allow_list, module_publishing_allowed
            }
        );
    }
    spec initialize {
        /// Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemTimestamp::AbortsIfNotGenesis;
        include DiemConfig::PublishNewConfigAbortsIf<DiemTransactionPublishingOption>;
        include DiemConfig::PublishNewConfigEnsures<DiemTransactionPublishingOption> {
            payload: DiemTransactionPublishingOption {
                script_allow_list, module_publishing_allowed
            }};
    }

    /// Check if sender can execute script with `hash`
    public fun is_script_allowed(account: &signer, hash: &vector<u8>): bool {
        // DiemRoot can send any script
        if (Roles::has_diem_root_role(account)) return true;

        // No one except DiemRoot can send scripts when transactions are halted
        if (transactions_halted()) return false;

        // The adapter passes an empty hash for script functions. All script functions are allowed
        if (Vector::is_empty(hash)) return true;

        let publish_option = DiemConfig::get<DiemTransactionPublishingOption>();
        // allowlist empty = open publishing, anyone can send txes
        Vector::is_empty(&publish_option.script_allow_list)
            // fixed allowlist. check inclusion
            || Vector::contains(&publish_option.script_allow_list, hash)
    }
    spec is_script_allowed {
        include
            !Roles::has_diem_root_role(account) && !transactions_halted() && !Vector::is_empty(hash)
            ==> DiemConfig::AbortsIfNotPublished<DiemTransactionPublishingOption>{};
    }
    spec schema AbortsIfNoTransactionPublishingOption {
        include DiemTimestamp::is_genesis() ==> DiemConfig::AbortsIfNotPublished<DiemTransactionPublishingOption>{};
    }

    /// Check if a sender can publish a module
    public fun is_module_allowed(account: &signer): bool {
        let publish_option = DiemConfig::get<DiemTransactionPublishingOption>();

        publish_option.module_publishing_allowed || Roles::has_diem_root_role(account)
    }
    spec is_module_allowed{
        include DiemConfig::AbortsIfNotPublished<DiemTransactionPublishingOption>{};
    }

    /// Allow the execution of arbitrary script or not.
    public fun set_open_script(dr_account: &signer) {
        Roles::assert_diem_root(dr_account);
        let publish_option = DiemConfig::get<DiemTransactionPublishingOption>();

        publish_option.script_allow_list = Vector::empty();
        DiemConfig::set<DiemTransactionPublishingOption>(dr_account, publish_option);
    }
    spec set_open_script {
        /// Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemConfig::AbortsIfNotPublished<DiemTransactionPublishingOption>;
        include DiemConfig::SetAbortsIf<DiemTransactionPublishingOption>{account: dr_account};
    }

    /// Allow module publishing from arbitrary sender or not.
    public fun set_open_module(dr_account: &signer, open_module: bool) {
        Roles::assert_diem_root(dr_account);

        let publish_option = DiemConfig::get<DiemTransactionPublishingOption>();

        publish_option.module_publishing_allowed = open_module;
        DiemConfig::set<DiemTransactionPublishingOption>(dr_account, publish_option);
    }
    spec set_open_module {
        /// Must abort if the signer does not have the DiemRoot role [[H11]][PERMISSION].
        include Roles::AbortsIfNotDiemRoot{account: dr_account};

        include DiemConfig::AbortsIfNotPublished<DiemTransactionPublishingOption>;
        include DiemConfig::SetAbortsIf<DiemTransactionPublishingOption>{account: dr_account};
    }

    /// If called, transactions cannot be sent from any account except DiemRoot
    public fun halt_all_transactions(dr_account: &signer) {
        Roles::assert_diem_root(dr_account);
        assert!(
            !exists<HaltAllTransactions>(Signer::address_of(dr_account)),
            Errors::already_published(EHALT_ALL_TRANSACTIONS),
        );
        move_to(dr_account, HaltAllTransactions {});
    }

    /// If called, transactions can be sent from any account once again
    public fun resume_transactions(dr_account: &signer) acquires HaltAllTransactions {
        Roles::assert_diem_root(dr_account);
        let dr_address = Signer::address_of(dr_account);
        assert!(
            exists<HaltAllTransactions>(dr_address),
            Errors::already_published(EHALT_ALL_TRANSACTIONS),
        );

        let HaltAllTransactions {} = move_from<HaltAllTransactions>(dr_address);
    }

    /// Return true if all non-administrative transactions are currently halted
    fun transactions_halted(): bool {
        exists<HaltAllTransactions>(@DiemRoot)
    }

    spec module { } // Switch documentation context to module level.

    /// # Initialization
    spec module {
        invariant [suspendable] DiemTimestamp::is_operating() ==>
            DiemConfig::spec_is_published<DiemTransactionPublishingOption>();
    }

    /// # Access Control

    /// Only `set_open_script`, and `set_open_module` can modify the
    /// DiemTransactionPublishingOption config [[H11]][PERMISSION]
    spec schema DiemVersionRemainsSame {
        ensures old(DiemConfig::spec_is_published<DiemTransactionPublishingOption>()) ==>
            global<DiemConfig<DiemTransactionPublishingOption>>(@DiemRoot) ==
                old(global<DiemConfig<DiemTransactionPublishingOption>>(@DiemRoot));
    }
    spec module {
        apply DiemVersionRemainsSame to * except set_open_script, set_open_module;
    }

    /// # Helper Functions
    spec module {
        fun spec_is_script_allowed(account: signer, hash: vector<u8>): bool {
            let publish_option = DiemConfig::spec_get_config<DiemTransactionPublishingOption>();
            Roles::has_diem_root_role(account) || (!transactions_halted() && (
                Vector::is_empty(hash) ||
                    (Vector::is_empty(publish_option.script_allow_list)
                        || contains(publish_option.script_allow_list, hash))
            ))
        }

        fun spec_is_module_allowed(account: signer): bool {
            let publish_option = DiemConfig::spec_get_config<DiemTransactionPublishingOption>();
            publish_option.module_publishing_allowed || Roles::has_diem_root_role(account)
        }
    }
}

address 0x1 {

/// This module defines a struct storing the publishing policies for the VM.
module LibraTransactionPublishingOption {
    use 0x1::Vector;
    use 0x1::LibraConfig::{Self, LibraConfig};
    use 0x1::LibraTimestamp;
    use 0x1::Errors;
    use 0x1::CoreAddresses;
    use 0x1::Roles;

    const SCRIPT_HASH_LENGTH: u64 = 32;

    /// The script hash has an invalid length
    const EINVALID_SCRIPT_HASH: u64 = 0;
    /// The script hash already exists in the allowlist
    const EALLOWLIST_ALREADY_CONTAINS_SCRIPT: u64 = 1;

    /// Defines and holds the publishing policies for the VM. There are three possible configurations:
    /// 1. No module publishing, only allow-listed scripts are allowed.
    /// 2. No module publishing, custom scripts are allowed.
    /// 3. Both module publishing and custom scripts are allowed.
    /// We represent these as the following resource.
    struct LibraTransactionPublishingOption {
        /// Only script hashes in the following list can be executed by the network. If the vector is empty, no
        /// limitation would be enforced.
        script_allow_list: vector<vector<u8>>,
        /// Anyone can publish new module if this flag is set to true.
        module_publishing_allowed: bool,
    }

    public fun initialize(
        lr_account: &signer,
        script_allow_list: vector<vector<u8>>,
        module_publishing_allowed: bool,
    ) {
        LibraTimestamp::assert_genesis();
        Roles::assert_libra_root(lr_account);

        LibraConfig::publish_new_config(
            lr_account,
            LibraTransactionPublishingOption {
                script_allow_list, module_publishing_allowed
            }
        );
    }
    spec fun initialize {
        /// Must abort if the signer does not have the LibraRoot role [[H11]][PERMISSION].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};

        include LibraTimestamp::AbortsIfNotGenesis;
        include LibraConfig::PublishNewConfigAbortsIf<LibraTransactionPublishingOption>;
        include LibraConfig::PublishNewConfigEnsures<LibraTransactionPublishingOption> {
            payload: LibraTransactionPublishingOption {
                script_allow_list, module_publishing_allowed
            }};
    }

    /// Check if sender can execute script with `hash`
    public fun is_script_allowed(account: &signer, hash: &vector<u8>): bool {
        let publish_option = LibraConfig::get<LibraTransactionPublishingOption>();

        Vector::is_empty(&publish_option.script_allow_list)
            || Vector::contains(&publish_option.script_allow_list, hash)
            || Roles::has_libra_root_role(account)
    }
    spec fun is_script_allowed {
        include AbortsIfNoTransactionPublishingOption;
    }
    spec schema AbortsIfNoTransactionPublishingOption {
        include LibraTimestamp::is_genesis() ==> LibraConfig::AbortsIfNotPublished<LibraTransactionPublishingOption>{};
    }

    /// Check if a sender can publish a module
    public fun is_module_allowed(account: &signer): bool {
        let publish_option = LibraConfig::get<LibraTransactionPublishingOption>();

        publish_option.module_publishing_allowed || Roles::has_libra_root_role(account)
    }
    spec fun is_module_allowed{
        include AbortsIfNoTransactionPublishingOption;
    }

    /// Add `new_hash` to the list of script hashes that is allowed to be executed by the network.
    public fun add_to_script_allow_list(lr_account: &signer, new_hash: vector<u8>) {
        Roles::assert_libra_root(lr_account);

        assert(Vector::length(&new_hash) == SCRIPT_HASH_LENGTH, Errors::invalid_argument(EINVALID_SCRIPT_HASH));

        let publish_option = LibraConfig::get<LibraTransactionPublishingOption>();
        if (Vector::contains(&publish_option.script_allow_list, &new_hash)) {
              abort Errors::invalid_argument(EALLOWLIST_ALREADY_CONTAINS_SCRIPT)
        };
        Vector::push_back(&mut publish_option.script_allow_list, new_hash);

        LibraConfig::set<LibraTransactionPublishingOption>(lr_account, publish_option);
    }
    spec fun add_to_script_allow_list {
        /// Must abort if the signer does not have the LibraRoot role [[H11]][PERMISSION].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};

        let allow_list = LibraConfig::get<LibraTransactionPublishingOption>().script_allow_list;
        aborts_if Vector::length(new_hash) != SCRIPT_HASH_LENGTH with Errors::INVALID_ARGUMENT;
        aborts_if Vector::spec_contains(allow_list, new_hash) with Errors::INVALID_ARGUMENT;
        include LibraConfig::AbortsIfNotPublished<LibraTransactionPublishingOption>;
        include LibraConfig::SetAbortsIf<LibraTransactionPublishingOption>{account: lr_account};
    }

    /// Allow the execution of arbitrary script or not.
    public fun set_open_script(lr_account: &signer) {
        Roles::assert_libra_root(lr_account);
        let publish_option = LibraConfig::get<LibraTransactionPublishingOption>();

        publish_option.script_allow_list = Vector::empty();
        LibraConfig::set<LibraTransactionPublishingOption>(lr_account, publish_option);
    }
    spec fun set_open_script {
        /// Must abort if the signer does not have the LibraRoot role [[H11]][PERMISSION].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};

        include LibraConfig::AbortsIfNotPublished<LibraTransactionPublishingOption>;
        include LibraConfig::SetAbortsIf<LibraTransactionPublishingOption>{account: lr_account};
    }

    /// Allow module publishing from arbitrary sender or not.
    public fun set_open_module(lr_account: &signer, open_module: bool) {
        Roles::assert_libra_root(lr_account);

        let publish_option = LibraConfig::get<LibraTransactionPublishingOption>();

        publish_option.module_publishing_allowed = open_module;
        LibraConfig::set<LibraTransactionPublishingOption>(lr_account, publish_option);
    }
    spec fun set_open_module {
        /// Must abort if the signer does not have the LibraRoot role [[H11]][PERMISSION].
        include Roles::AbortsIfNotLibraRoot{account: lr_account};

        include LibraConfig::AbortsIfNotPublished<LibraTransactionPublishingOption>;
        include LibraConfig::SetAbortsIf<LibraTransactionPublishingOption>{account: lr_account};
    }

    spec module { } // Switch documentation context to module level.

    /// # Initialization
    spec module {
        invariant [global] LibraTimestamp::is_operating() ==>
            LibraConfig::spec_is_published<LibraTransactionPublishingOption>();
    }

    /// # Access Control

    /// Only `add_to_script_allow_list`, `set_open_script`, and `set_open_module` can modify the
    /// LibraTransactionPublishingOption config [[H11]][PERMISSION]
    spec schema LibraVersionRemainsSame {
        ensures old(LibraConfig::spec_is_published<LibraTransactionPublishingOption>()) ==>
            global<LibraConfig<LibraTransactionPublishingOption>>(CoreAddresses::LIBRA_ROOT_ADDRESS()) ==
                old(global<LibraConfig<LibraTransactionPublishingOption>>(CoreAddresses::LIBRA_ROOT_ADDRESS()));
    }
    spec module {
        apply LibraVersionRemainsSame to * except add_to_script_allow_list, set_open_script, set_open_module;
    }

    /// # Helper Functions
    spec module {
        define spec_is_script_allowed(account: signer, hash: vector<u8>): bool {
            let publish_option = LibraConfig::spec_get_config<LibraTransactionPublishingOption>();
            Vector::is_empty(publish_option.script_allow_list)
                || Vector::spec_contains(publish_option.script_allow_list, hash)
                || Roles::has_libra_root_role(account)
        }

        define spec_is_module_allowed(account: signer): bool {
            let publish_option = LibraConfig::spec_get_config<LibraTransactionPublishingOption>();
            publish_option.module_publishing_allowed || Roles::has_libra_root_role(account)
        }
    }
}
}

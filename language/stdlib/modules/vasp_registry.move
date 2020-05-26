address 0x0 {

module VASPRegistry {
    use 0x0::LibraConfig;
    use 0x0::Signer;
    use 0x0::Transaction;
    use 0x0::Vector;

    // An on-chain config holding all of the registered vasps.
    struct VASPRegistry {
        registered_vasps: vector<address>
    }

    // An operations capability to allow updating of the on-chain config
    resource struct VASPRegistrationCapability {
        cap: LibraConfig::ModifyConfigCapability<VASPRegistry>,
    }

    public fun initialize(config_account: &signer): VASPRegistrationCapability {
        // enforce that this is only going to one specific address,
        Transaction::assert(
            Signer::address_of(config_account) == LibraConfig::default_config_address(),
            0
        );
        let cap = LibraConfig::publish_new_config_with_capability(
                     VASPRegistry {
                         registered_vasps: Vector::empty(),
                     },
                     config_account
                  );

        VASPRegistrationCapability { cap }
    }

    public fun registered_vasp_addresses(): vector<address> {
        let config = LibraConfig::get<VASPRegistry>();
        *&config.registered_vasps
    }

    public fun remove_vasp(
        parent_vasp_addr: address,
        cap: &VASPRegistrationCapability,
    ) {
        let config = LibraConfig::get<VASPRegistry>();
        let (found, index) = Vector::index_of(&config.registered_vasps, &parent_vasp_addr);
        if (found) { let _ = Vector::swap_remove(&mut config.registered_vasps, index) };
        LibraConfig::set_with_capability(&cap.cap, config);
    }

    public fun add_vasp(
        parent_vasp_addr: address,
        cap: &VASPRegistrationCapability,
    ) {
        let config = LibraConfig::get<VASPRegistry>();
        Vector::push_back(&mut config.registered_vasps, parent_vasp_addr);
        LibraConfig::set_with_capability(&cap.cap, config);
    }
}
}

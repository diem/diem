module ConfigHolder {
    use 0x0::LibraConfig;
    resource struct Holder<T> {
        cap: LibraConfig::ModifyConfigCapability<T>
    }

    public fun hold<T>(cap: LibraConfig::ModifyConfigCapability<T>) {
        move_to_sender(Holder<T>{ cap })
    }

    public fun get<T>(): LibraConfig::ModifyConfigCapability<T>
    acquires Holder {
        let Holder<T>{ cap } = move_from<Holder<T>>({{association}});
        cap
    }
}

//! new-transaction
script {
    use 0x0::LibraConfig;
    fun main() {
        LibraConfig::initialize_configuration();
    }
}
// check: ABORTED
// check: 1

//! new-transaction
script {
    use 0x0::LibraConfig;
    fun main() {
        let _x = LibraConfig::get<u64>();
    }
}
// check: ABORTED
// check: 24

//! new-transaction
script {
    use 0x0::LibraConfig;
    fun main() {
        LibraConfig::set(0);
    }
}
// check: ABORTED
// check: 24

//! new-transaction
script {
    use 0x0::LibraConfig;
    use {{default}}::ConfigHolder;
    fun main() {
        ConfigHolder::hold(
            LibraConfig::publish_new_config_with_capability(0)
        );
    }
}
// check: ABORTED
// check: 1

//! new-transaction
//! sender: association
script {
    use 0x0::LibraConfig;
    use {{default}}::ConfigHolder;
    fun main() {
        ConfigHolder::hold(
            LibraConfig::publish_new_config_with_capability<u64>(0)
        );
    }
}

//! new-transaction
script {
    use 0x0::LibraConfig;
    use {{default}}::ConfigHolder;
    fun main() {
        let cap = ConfigHolder::get<u64>();
        LibraConfig::set_with_capability(&cap, 0);
        ConfigHolder::hold(cap);
    }
}
// check: ABORTED
// check: 24

//! new-transaction
script {
    use 0x0::LibraConfig;
    fun main() {
        LibraConfig::publish_new_config(0)
    }
}
// check: ABORTED
// check: 1

//! new-transaction
script {
    use 0x0::LibraConfig;
    fun main() {
        LibraConfig::publish_new_config_with_delegate(0, {{association}})
    }
}
// check: ABORTED
// check: 1

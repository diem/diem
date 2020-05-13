//! account: bob, 100000LBR, 0, unhosted

//! new-transaction
//! sender: bob
script {
    use 0x0::Association;
    fun main() {
        Association::apply_for_association();
    }
}

//! new-transaction
//! sender: association
script {
    use 0x0::Association;
    fun main() {
        Association::grant_association_address({{bob}});
    }
}

//! new-transaction
//! sender: bob
script {
    use 0x0::VASP;
    fun main() {
        VASP::initialize();
    }
}
// check: ABORTED
// check: 7000

//! new-transaction
//! sender: association
script {
    use 0x0::VASP;
    use 0x0::Association;
    fun main() {
        Association::apply_for_privilege<VASP::CreationPrivilege>();
        Association::grant_privilege<VASP::CreationPrivilege>({{association}});
    }
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
    use 0x0::VASP;
    fun main() {
        VASP::recertify_vasp({{bob}});
    }
}
// check: ABORTED
// check: 7001

//! new-transaction
//! sender: association
script {
    use 0x0::VASP;
    fun main() {
        VASP::decertify_vasp({{bob}});
    }
}
// check: ABORTED
// check: 7001

//! new-transaction
//! sender: association
script {
    use 0x0::VASP;
    use 0x0::Testnet;
    fun main() {
        Testnet::remove_testnet();
        let _ = VASP::create_root_vasp_credential(x"", x"", x"");
        Testnet::initialize();
    }
}
// check: ABORTED
// check: 10041

//! new-transaction
script {
    use 0x0::VASP;
    fun main() {
        VASP::apply_for_vasp_root_credential(x"", x"", x"");
    }
}
// check: ABORTED
// check: 7004

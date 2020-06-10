// A module providing functionality to the script*.move tests
address 0x1 {

// TODO: This file gets errors for reasons I do not understand.

module ScriptProvider {
    use 0x1::Signer;

    spec module {
        pragma verify = true;
    }


    resource struct Info<T> {}

    public fun register<T>(account: &signer) {
        assert(Signer::address_of(account) == 0x1, 1);
        move_to(account, Info<T>{})
    }
    spec schema RegisterConditions<T> {
        account: signer;
        aborts_if Signer::get_address(account) != 0x1;
        aborts_if exists<Info<T>>(0x1);
        ensures exists<Info<T>>(0x1);
    }
    spec fun register {
        include RegisterConditions<T>;
    }
}

}

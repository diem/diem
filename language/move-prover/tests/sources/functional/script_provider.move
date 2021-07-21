// A module providing functionality to the script*.move tests
address 0x1 {


module ScriptProvider {
    use Std::Signer;

    spec module {
        // TODO: This file gets errors for reasons I do not understand.
        // The errors are produced non-deterministically, therefore turned off.
        pragma verify = false;
    }


    struct Info<phantom T> has key {}

    public fun register<T: store>(account: &signer) {
        assert(Signer::address_of(account) == @0x1, 1);
        move_to(account, Info<T>{})
    }
    spec schema RegisterConditions<T> {
        account: signer;
        aborts_if Signer::spec_address_of(account) != @0x1;
        aborts_if exists<Info<T>>(@0x1);
        ensures exists<Info<T>>(@0x1);
    }
    spec register {
        include RegisterConditions<T>;
    }
}

}

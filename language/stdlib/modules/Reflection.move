address 0x0 {
module Reflection {
    /// Return module address, module name, and type name of `E`.
    native public fun name_of<E>(): (address, vector<u8>, vector<u8>);
}
}
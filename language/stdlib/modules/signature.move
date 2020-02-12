address 0x0:

module Signature {
    native public fun ed25519_verify(signature: bytearray, public_key: bytearray, message: bytearray): bool;
    native public fun ed25519_threshold_verify(bitmap: bytearray, signature: bytearray, public_key: bytearray, message: bytearray): u64;
}

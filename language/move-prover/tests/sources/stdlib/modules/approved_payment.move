// dep: tests/sources/stdlib/modules/libra.move
// dep: tests/sources/stdlib/modules/libra_account.move
// dep: tests/sources/stdlib/modules/signature.move
// dep: tests/sources/stdlib/modules/transaction.move
// dep: tests/sources/stdlib/modules/vector.move
// dep: tests/sources/stdlib/modules/lcs.move
// dep: tests/sources/stdlib/modules/lbr.move
// dep: tests/sources/stdlib/modules/libra_transaction_timeout.move
// dep: tests/sources/stdlib/modules/libra_time.move
// dep: tests/sources/stdlib/modules/hash.move
// no-verify

// Module that allows a payee to approve payments with a cryptographic signature. The basic flow is:
// (1) Payer sends `metadata` to the payee
// (2) Payee signs `metadata` and sends 64 byte signature back to the payer
// (3) Payer sends an approved payment to the payee by sending a transaction invoking `deposit`
//     with payment metadata + signature. The transaction will abort if the signature check fails.
// Note: approved payments are an accounting convenience/a courtesy mechansim for the payee, *not*
// a hurdle that must be cleared for all payments to the payee. In addition, approved payments do
// not have replay protection.
address 0x0:
module ApprovedPayment {
    use 0x0::Libra;
    use 0x0::LibraAccount;
    use 0x0::Signature;
    use 0x0::Transaction;
    use 0x0::Vector;

    // A resource to be published under the payee's account
    resource struct T {
        // 32 byte single Ed25519 public key whose counterpart must be used to sign the payment
        // metadata. Note that this is different (and simpler) than the `authentication_key` used in
        // LibraAccount::T, which is a hash of a public key + signature scheme identifier.
        public_key: vector<u8>,
        // TODO: events?
    }

    // Deposit `coin` in `payee`'s account if the `signature` on the payment metadata matches the
    // public key stored in the `approved_payment` resource
    public fun deposit<Token>(
        approved_payment: &T,
        payee: address,
        coin: Libra::T<Token>,
        metadata: vector<u8>,
        signature: vector<u8>
    ) {
        // Sanity check of signature validity
        Transaction::assert(Vector::length(&signature) == 64, 9001); // TODO: proper error code
        // Cryptographic check of signature validity
        Transaction::assert(
            Signature::ed25519_verify(
                signature,
                *&approved_payment.public_key,
                copy metadata
            ),
            9002, // TODO: proper error code
        );
        LibraAccount::deposit_with_metadata<Token>(payee, coin, metadata)
    }

    // Wrapper of `deposit` that withdraw's from the sender's balance and uses the top-level
    // `ApprovedPayment::T` resource under the payee account.
    public fun deposit_to_payee<Token>(
        payee: address,
        amount: u64,
        metadata: vector<u8>,
        signature: vector<u8>
    ) acquires T {
        deposit<Token>(
            borrow_global<T>(payee),
            payee,
            LibraAccount::withdraw_from_sender<Token>(amount),
            metadata,
            signature
        )
    }

    // Rotate the key used to sign approved payments. This will invalidate any approved payments
    // that are currently in flight
    public fun rotate_key(approved_payment: &mut T, new_public_key: vector<u8>) {
        approved_payment.public_key = new_public_key
    }

    // Wrapper of `rotate_key` that rotates the sender's key
    public fun rotate_sender_key(new_public_key: vector<u8>) acquires T {
        // Sanity check for key validity
        Transaction::assert(Vector::length(&new_public_key) == 32, 9003); // TODO: proper error code
        rotate_key(borrow_global_mut<T>(Transaction::sender()), new_public_key)
    }

    // Publish an ApprovedPayment::T resource under the sender's account with approval key
    // `public_key`
    public fun publish(public_key: vector<u8>) {
        // Sanity check for key validity
        Transaction::assert(Vector::length(&public_key) == 32, 9003); // TODO: proper error code
        move_to_sender(T { public_key })
    }

    // Remove and destroy the ApprovedPayment::T resource under the sender's account
    public fun unpublish_from_sender() acquires T {
        let T { public_key: _ } = move_from<T>(Transaction::sender())
    }

    // Return true if an ApprovedPayment::T resource exists under `addr`
    public fun exists(addr: address): bool {
        ::exists<T>(addr)
    }

}

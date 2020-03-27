// dep: tests/sources/stdlib/modules/transaction.move
address 0x0:

// TODO: add optional timeout for reclaiming by original publisher once we have implemented time
module Offer {
  use 0x0::Transaction;
  // A wrapper around value `offered` that can be claimed by the address stored in `for`.
  resource struct T<Offered> { offered: Offered, for: address }

  // Publish a value of type `Offered` under the sender's account. The value can be claimed by
  // either the `for` address or the transaction sender.
  public fun create<Offered>(offered: Offered, for: address) {
    move_to_sender<T<Offered>>(T<Offered> { offered: offered, for: for });
  }

  // Claim the value of type `Offered` published at `offer_address`.
  // Only succeeds if the sender is the intended recipient stored in `for` or the original
  // publisher `offer_address`.
  // Also fails if no such value exists.
  public fun redeem<Offered>(offer_address: address): Offered acquires T {
    let T<Offered> { offered, for } = move_from<T<Offered>>(offer_address);
    let sender = Transaction::sender();
    // fail with INSUFFICIENT_PRIVILEGES
    Transaction::assert(sender == for || sender == offer_address, 11);
    offered
  }

  // Returns true if an offer of type `Offered` exists at `offer_address`.
  public fun exists_at<Offered>(offer_address: address): bool {
    exists<T<Offered>>(offer_address)
  }

  // Returns the address of the `Offered` type stored at `offer_address.
  // Fails if no such `Offer` exists.
  public fun address_of<Offered>(offer_address: address): address acquires T {
    borrow_global<T<Offered>>(offer_address).for
  }
}

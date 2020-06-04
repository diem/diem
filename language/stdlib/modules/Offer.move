address 0x0 {

// TODO: add optional timeout for reclaiming by original publisher once we have implemented time
module Offer {
  use 0x0::Signer;
  use 0x0::Transaction;
  // A wrapper around value `offered` that can be claimed by the address stored in `for`.
  resource struct Offer<Offered> { offered: Offered, for: address }

  // Publish a value of type `Offered` under the sender's account. The value can be claimed by
  // either the `for` address or the transaction sender.
  public fun create<Offered>(account: &signer, offered: Offered, for: address) {
    move_to(account, Offer<Offered> { offered, for });
  }

  // Claim the value of type `Offered` published at `offer_address`.
  // Only succeeds if the sender is the intended recipient stored in `for` or the original
  // publisher `offer_address`.
  // Also fails if no such value exists.
  public fun redeem<Offered>(account: &signer, offer_address: address): Offered acquires Offer {
    let Offer<Offered> { offered, for } = move_from<Offer<Offered>>(offer_address);
    let sender = Signer::address_of(account);
    // fail with INSUFFICIENT_PRIVILEGES
    Transaction::assert(sender == for || sender == offer_address, 11);
    offered
  }

  // Returns true if an offer of type `Offered` exists at `offer_address`.
  public fun exists_at<Offered>(offer_address: address): bool {
    exists<Offer<Offered>>(offer_address)
  }

  // Returns the address of the `Offered` type stored at `offer_address.
  // Fails if no such `Offer` exists.
  public fun address_of<Offered>(offer_address: address): address acquires Offer {
    borrow_global<Offer<Offered>>(offer_address).for
  }
}

}

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

  // **************** SPECIFICATIONS ****************

  /*
  This module defines a resource T that is used as a permissioned trading scheme between accounts.
  It defines two main functions for creating and retrieving a struct offered by some user
  inside the resource T under the offerer's account.

  Currently, the only other module that depends on this module is LibraConfig, where it's used to
  pass a capability to an account that allows it to modify a config.
  */

  /// # Module specification

  spec module {
    /// Verify all functions in this module
    pragma verify = true;
    /// Helper function that returns whether or not the `recipient` is an intended
    /// recipient of the offered struct in the T<Offered> resource at the address `offer_address`
    define is_offer_recipient<Offered>(offer_addr: address, recipient: address): bool {
      recipient == global<T<Offered>>(offer_addr).for || recipient == offer_addr
    }
  }

  // Switch documentation context back to module level.
  spec module {}

  /// ## Creation of Offers

  spec schema OnlyCreateCanCreateOffer {
    /// Only `Self::create` can create an Offer.T under an address.
    ///
    /// **Informally:** No function can create an offer. If there didn't
    /// exist an Offer under some `addr`, then it continues not to have one.
    ensures all(domain<type>(), |ty|
              all(domain<address>(), |addr|
                !old(exists<T<ty>>(addr)) ==> !exists<T<ty>>(addr)));
  }
  spec module {
    /// Apply OnlyCreateCanCreateOffer
    apply OnlyCreateCanCreateOffer to *<Offered>, * except create;
  }
  spec fun create {
    /// Offer a struct to the account under address `for` by
    /// placing the Offer under the signer's address
    aborts_if exists<T<Offered>>(Signer::get_address(account));
    ensures exists<T<Offered>>(Signer::get_address(account));
    ensures global<T<Offered>>(Signer::get_address(account)) == T<Offered> { offered: offered, for: for };
  }

  // Switch documentation context back to module level.
  spec module {}

  /// ## Removal of Offers

  spec schema OnlyRedeemCanRemoveOffer {
    /// Only `Self::redeem` can remove an Offer.T.
    ///
    /// **Informally:** No other function except for `redeem` can remove an Offer from an account
    ensures all(domain<type>(), |ty|
              all(domain<address>(), |addr|
                old(exists<T<ty>>(addr))
                  ==> (exists<T<ty>>(addr) && old(global<T<ty>>(addr)) == global<T<ty>>(addr))));
  }
  spec module {
    /// Show that every function except `Self::redeem` can remove an Offer.T from the global store
    apply OnlyRedeemCanRemoveOffer to *<Offered>, * except redeem;
  }
  spec fun redeem {
    /// **Informally:** Redeems an offer (T) under the account at `offer_address`
    aborts_if !exists<T<Offered>>(offer_address);
    aborts_if !is_offer_recipient<Offered>(offer_address, Signer::get_address(account));
    ensures old(exists<T<Offered>>(offer_address)) && !exists<T<Offered>>(offer_address);
    ensures result == old(global<T<Offered>>(offer_address).offered);
  }

  // Switch documentation context back to module level.
  spec module {}

  spec fun exists_at {
    /// Returns whether or not an offer (T) is under the given address `offer_address`
    ensures result == exists<T<Offered>>(offer_address);
  }

  spec fun address_of {
    /// Returns the address of the intended recipient of the Offer under
    /// the `offer_address` if one exists
    aborts_if !exists<T<Offered>>(offer_address);
    ensures result == global<T<Offered>>(offer_address).for;
  }
}

}

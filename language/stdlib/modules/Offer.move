address 0x1 {

// TODO: add optional timeout for reclaiming by original publisher once we have implemented time
module Offer {
  use 0x1::Signer;
  // A wrapper around value `offered` that can be claimed by the address stored in `for`.
  resource struct Offer<Offered> { offered: Offered, for: address }

  const EOFFER_DNE_FOR_ACCOUNT: u64 = 0;

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
    assert(sender == for || sender == offer_address, EOFFER_DNE_FOR_ACCOUNT);
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

  /// # Module specification

  /**
  This module defines a resource `Offer` that is used as a permissioned trading scheme between accounts.
  It defines two main functions for creating and retrieving a struct offered by some user
  inside the resource `Offer` under the offerer's account.

  Currently, the only other module that depends on this module is LibraConfig, where it's used to
  pass a capability to an account that allows it to modify a config.
  */
  spec module {
    /// Verify all functions in this module
    pragma verify = true;
    /// Helper function that returns whether or not the `recipient` is an intended
    /// recipient of the offered struct in the `Offer<Offered>` resource at the address `offer_address`
    /// Returns true if the recipient is allowed to redeem `Offer<Offered>` at `offer_address`
    /// and false otherwise.
    ///
    /// TODO (dd): this is undefined if the offer does not exist. Should this be anded with
    /// "exists_at"?

    define is_allowed_recipient<Offered>(offer_addr: address, recipient: address): bool {
      recipient == global<Offer<Offered>>(offer_addr).for || recipient == offer_addr
    }

    /// Mirrors the Move function exists_at<Offered>, above.
    define spec_exists_at<Offered>(offer_addr: address): bool {
        exists<Offer<Offered>>(offer_addr)
    }

  }

  /// ## Creation of Offers

  spec schema OnlyCreateCanCreateOffer {
    /// Only `Self::create` can create a resource `Offer` under an address.
    ///
    /// **Informally:** No function to which this is applied can create an offer.
    /// If there didn't exist an offer under some `addr`, then it continues
    /// not to have one.
    ensures forall ty: type, addr: address where !old(exists<Offer<ty>>(addr)) : !exists<Offer<ty>>(addr);
  }
  spec module {
    /// Apply OnlyCreateCanCreateOffer
    apply OnlyCreateCanCreateOffer to *<Offered>, * except create;
  }
  spec fun create {
    /// Offer a struct to the account under address `for` by
    /// placing the offer under the signer's address
    aborts_if exists<Offer<Offered>>(Signer::spec_address_of(account));
    ensures exists<Offer<Offered>>(Signer::spec_address_of(account));
    ensures global<Offer<Offered>>(Signer::spec_address_of(account)) == Offer<Offered> { offered: offered, for: for };
  }

  // Switch documentation context back to module level.
  spec module {}

  /// ## Removal of Offers

  spec schema OnlyRedeemCanRemoveOffer {
    /// Only `Self::redeem` can remove the `Offer` resource from an account.
    ///
    /// **Informally:** No other function except for `redeem` can remove an offer from an account.
    ensures forall ty: type, addr: address where old(exists<Offer<ty>>(addr)) :
              (exists<Offer<ty>>(addr) && global<Offer<ty>>(addr) == old(global<Offer<ty>>(addr)));
  }
  spec module {
    /// Enforce that every function except `Self::redeem` can remove an offer from the global store.
    apply OnlyRedeemCanRemoveOffer to *<Offered>, * except redeem;
  }
  spec fun redeem {
    /// Aborts if there is no offer under `offer_address` or if the account
    /// cannot redeem the offer.
    /// Ensures that the offered struct under `offer_address` is removed is returned.
    aborts_if !exists<Offer<Offered>>(offer_address);
    aborts_if !is_allowed_recipient<Offered>(offer_address, Signer::spec_address_of(account));
    ensures old(exists<Offer<Offered>>(offer_address)) && !exists<Offer<Offered>>(offer_address);
    ensures result == old(global<Offer<Offered>>(offer_address).offered);
  }

  // Switch documentation context back to module level.
  spec module {}

  spec fun exists_at {
    /// Returns whether or not an `Offer` resource is under the given address `offer_address`.
    ensures result == exists<Offer<Offered>>(offer_address);
  }

  spec fun address_of {
    /// Aborts is there is no offer resource `Offer` at the `offer_address`.
    /// Returns the address of the intended recipient of the Offer
    /// under the `offer_address`.
    aborts_if !exists<Offer<Offered>>(offer_address);
    ensures result == global<Offer<Offered>>(offer_address).for;
  }
}

}

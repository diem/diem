/// Provides a way to transfer structs from one account to another in two transactions.
/// Unlike many languages, Move cannot move data from one account to another with
/// single-signer transactions. As of this writing, ordinary transactions can only have
/// a single signer, and Move code can only store to an address (via `move_to`) if it
/// can supply a reference to a signer for the destination address (there are special case
/// exceptions in Genesis and DiemAccount where there can temporarily be multiple signers).
///
/// Offer solves this problem by providing an `Offer` resource.  To move a struct `T` from
/// account A to B, account A first publishes an `Offer<T>` resource at `address_of(A)`,
/// using the `Offer::create` function.
/// Then account B, in a separate transaction, can move the struct `T` from the `Offer` at
/// A's address to the desired destination. B accesses the resource using the `redeem` function,
/// which aborts unless the `for` field is B's address (preventing other addresses from
/// accessing the `T` that is intended only for B). A can also redeem the `T` value if B hasn't
/// redeemed it.
module Std::Offer {
  use Std::Signer;
  use Std::Errors;

  /// A wrapper around value `offered` that can be claimed by the address stored in `for`.
  struct Offer<Offered> has key { offered: Offered, for: address }

  /// An offer of the specified type for the account does not exist
  const EOFFER_DNE_FOR_ACCOUNT: u64 = 0;

  /// Address already has an offer of this type.
  const EOFFER_ALREADY_CREATED: u64 = 1;

  /// Address does not have an offer of this type to redeem.
  const EOFFER_DOES_NOT_EXIST: u64 = 2;

  /// Publish a value of type `Offered` under the sender's account. The value can be claimed by
  /// either the `for` address or the transaction sender.
  public fun create<Offered: store>(account: &signer, offered: Offered, for: address) {
    assert(!exists<Offer<Offered>>(Signer::address_of(account)), Errors::already_published(EOFFER_ALREADY_CREATED));
    move_to(account, Offer<Offered> { offered, for });
  }
  spec create {
    /// Offer a struct to the account under address `for` by
    /// placing the offer under the signer's address
    aborts_if exists<Offer<Offered>>(Signer::spec_address_of(account))
        with Errors::ALREADY_PUBLISHED;
    ensures exists<Offer<Offered>>(Signer::spec_address_of(account));
    ensures global<Offer<Offered>>(Signer::spec_address_of(account)) == Offer<Offered> { offered: offered, for: for };
  }

  /// Claim the value of type `Offered` published at `offer_address`.
  /// Only succeeds if the sender is the intended recipient stored in `for` or the original
  /// publisher `offer_address`.
  /// Also fails if there is no `Offer<Offered>` published.
  public fun redeem<Offered: store>(account: &signer, offer_address: address): Offered acquires Offer {
    assert(exists<Offer<Offered>>(offer_address), Errors::not_published(EOFFER_DOES_NOT_EXIST));
    let Offer<Offered> { offered, for } = move_from<Offer<Offered>>(offer_address);
    let sender = Signer::address_of(account);
    assert(sender == for || sender == offer_address, Errors::invalid_argument(EOFFER_DNE_FOR_ACCOUNT));
    offered
  }
  spec redeem {
    /// Aborts if there is no offer under `offer_address` or if the account
    /// cannot redeem the offer.
    /// Ensures that the offered struct under `offer_address` is removed.
    aborts_if !exists<Offer<Offered>>(offer_address)
        with Errors::NOT_PUBLISHED;
    aborts_if !is_allowed_recipient<Offered>(offer_address, Signer::spec_address_of(account))
        with Errors::INVALID_ARGUMENT;
    ensures !exists<Offer<Offered>>(offer_address);
    ensures result == old(global<Offer<Offered>>(offer_address).offered);
  }

  // Returns true if an offer of type `Offered` exists at `offer_address`.
  public fun exists_at<Offered: store>(offer_address: address): bool {
    exists<Offer<Offered>>(offer_address)
  }
  spec exists_at {
    aborts_if false;
    /// Returns whether or not an `Offer` resource is under the given address `offer_address`.
    ensures result == exists<Offer<Offered>>(offer_address);
  }


  // Returns the address of the `Offered` type stored at `offer_address.
  // Fails if no such `Offer` exists.
  public fun address_of<Offered: store>(offer_address: address): address acquires Offer {
    assert(exists<Offer<Offered>>(offer_address), Errors::not_published(EOFFER_DOES_NOT_EXIST));
    borrow_global<Offer<Offered>>(offer_address).for
  }
  spec address_of {
    /// Aborts is there is no offer resource `Offer` at the `offer_address`.
    /// Returns the address of the intended recipient of the Offer
    /// under the `offer_address`.
    aborts_if !exists<Offer<Offered>>(offer_address) with Errors::NOT_PUBLISHED;
    ensures result == global<Offer<Offered>>(offer_address).for;
  }

// =================================================================
// Module Specification

  spec module {} // switch documentation context back to module level

  /// # Access Control

  /// ## Creation of Offers

  spec schema NoOfferCreated<Offered> {
    /// Says no offer is created for any address. Later, it is applied to all functions
    /// except `create`
    ensures forall addr: address where !old(exists<Offer<Offered>>(addr)) : !exists<Offer<Offered>>(addr);
  }
  spec module {
    /// Apply OnlyCreateCanCreateOffer to every function except `create`
    apply NoOfferCreated<Offered> to *<Offered>, * except create;
  }

  /// ## Removal of Offers

  spec schema NoOfferRemoved<Offered> {
    /// Says no offer is removed for any address. Applied below to everything except `redeem`
    ensures forall addr: address where old(exists<Offer<Offered>>(addr)) :
              (exists<Offer<Offered>>(addr) && global<Offer<Offered>>(addr) == old(global<Offer<Offered>>(addr)));
  }
  spec module {
    /// Only `redeem` can remove an offer from the global store.
    apply NoOfferRemoved<Offered> to *<Offered>, * except redeem;
  }

  /// # Helper Functions

  spec module {
    /// Returns true if the recipient is allowed to redeem `Offer<Offered>` at `offer_address`
    /// and false otherwise.
    fun is_allowed_recipient<Offered>(offer_addr: address, recipient: address): bool {
      recipient == global<Offer<Offered>>(offer_addr).for || recipient == offer_addr
    }
  }



}

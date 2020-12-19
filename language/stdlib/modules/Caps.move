address 0x1 {
module Caps {
use 0x1::Signer;

/// Granted to an authority responsible for issuing and revoking `Cap<T>`'s
resource struct CapIssuer<T> {
  t: T,
  /// Starts at 1. all caps with a validity_timestamp >= time are valid, all others are invalid
  time: u64,
  /// If true, the issuer can revoke issued Cap<T>'s
  revocable: bool,
  /// If true, the holder can transfer its Cap<T> elsewhere
  transferrable: bool,
}

/// A non-forgeable capability representing the authority to do `T`
resource struct Cap<T> {
  /// Authority that issued the capability
  issuer: address,
  /// Timestamp that should be checked against `CapIssuer<T>.time` before issuing
  /// `EphemeralCap<T>`'s
  validity_timestamp: u64
}

/// Proof that `holder` holds a capability granted by `issuer`. A sensitive function `f` that should
/// only be accessed by someone with `T` privileges should indicate this via a signature like
/// `f(cap: &EphemeralCap<T>)` (e.g., `mint(amount: u64, cap: &EphemeralCap<Mint>)`
/// In a version of Move with constraints, it would be very important to omit `store` here to make
/// this into a hot potato (i.e. `drop copy struct EphemeralCap<T>`).
struct EphemeralCap<T> {
  issuer: address,
  holder: address,
}

/// EphemeralCap<T>'s can only be created by an account with a valid `Cap<T>`
spec struct EphemeralCap {
    // TODO: prover doesn't let data invariants depend on global state, so not sure how to state this
    // TODO: ideally, this would be an `invariant pack` --it's ok if the Cap<T> is revoked after the
    // EphemeralCap<T> is created.
    invariant !is_revoked<T>(issuer);
}

// === Specs ===

/// Every Cap<T> has a corresponding CapIssuer<T>
spec module {
    invariant [global]
        forall a: address, ty: type
        where exists<Cap<ty>>(a):
        exists<CapIssuer<ty>>(global<Cap<ty>>(a).issuer);
}

/// Non-transferrable Cap<T>'s cannot move across addresses
spec module {
    pragma verify = false; // TODO: this times out
    invariant update [global]
        forall a: address, ty: type
        where old(exists<Cap<ty>>(a)) && !get_cap_issuer<ty>(a).transferrable:
        exists<Cap<ty>>(a);
}

spec module {
    define get_cap_issuer<T>(a: address): CapIssuer<T> {
        global<CapIssuer<T>>(global<Cap<T>>(a).issuer)
    }
}

// === Functionality for a issuer of a Cap ===

/// Create a CapIssuer that can issue and revoke capabilities of type `T`.
/// Privileged creation of `T` should be managed in module that creates `T`
public fun create<T>(issuer: &signer, t: T, revocable: bool, transferrable: bool) {
  move_to(issuer, CapIssuer { t, time: 1, revocable, transferrable })
}

/// Grant a non-copyable Cap<T> to `recipient`
public fun grant<T>(issuer: &signer, recipient: &signer) acquires CapIssuer {
  let cap_issuer = borrow_global_mut<CapIssuer<T>>(Signer::address_of(issuer));
  let validity_timestamp = cap_issuer.time;
  move_to(
    recipient,
    Cap<T> { issuer: Signer::address_of(issuer), validity_timestamp }
  )
}

/// Revoke all outstanding Cap<T>'s
public fun revoke_all<T>(issuer: &signer) acquires CapIssuer {
  let cap_issuer = borrow_global_mut<CapIssuer<T>>(Signer::address_of(issuer));
  assert(cap_issuer.revocable, 0);
  cap_issuer.time = cap_issuer.time + 1
}

/// Revoke the Cap<T> under `to_revoke`
public fun revoke_one<T: copyable>(issuer: &signer, to_revoke: address) acquires Cap, CapIssuer {
  let cap_issuer = borrow_global_mut<CapIssuer<T>>(Signer::address_of(issuer));
  assert(cap_issuer.revocable, 0);
  let Cap { issuer: _, validity_timestamp: _ } = move_from<Cap<T>>(to_revoke);
}

// === Functionality for a holder of a Cap ===

/// Transfer the capability from `owner` to `recipient`
public fun transfer<T>(owner: &signer, recipient: &signer) acquires Cap, CapIssuer {
  let cap = move_from<Cap<T>>(Signer::address_of(owner));
  let cap_issuer = borrow_global<CapIssuer<T>>(cap.issuer);
  // make sure transfers of cap are allowed
  assert(cap_issuer.transferrable, 0);

  move_to(recipient, cap);
}

/// Create an EphemeralCap<T> using the `Cap<T>` published under `account`
public fun create_ephemeral<T>(account: &signer): EphemeralCap<T> acquires Cap, CapIssuer {
    let holder = Signer::address_of(account);
    let cap = borrow_global<Cap<T>>(holder);
    let issuer = cap.issuer;
    let cap_issuer = borrow_global<CapIssuer<T>>(issuer);
    // can't issue EphemeralCap's if the factory is expired
    assert(cap_issuer.time >= cap.validity_timestamp, 0);

    EphemeralCap<T> { issuer, holder }
}

// === Public functionality ===

/// Return the issuer of the Cap<T> published under a
public fun get_issuer<T>(a: address): address acquires Cap {
    let cap = borrow_global<Cap<T>>(a);
    cap.issuer
}

/// Return true if `a` has a revoked Cap<T>
public fun is_revoked<T>(a: address): bool acquires Cap, CapIssuer {
    let cap = borrow_global<Cap<T>>(a);
    let cap_issuer = borrow_global<CapIssuer<T>>(cap.issuer);
    cap_issuer.time >= cap.validity_timestamp
}

}
}

//! account: ta, 1000000, 0, true
//! account: tb, 1000000, 0, true
//! account: txa, 1000000, 0, true
//! account: txb, 1000000, 0, true
//! account: alice, 1000000, 0, true
//! account: vivian, 1000000, 0, true
//! account: bob, 1000000, 0, true
//! account: alex, 1000000, 0, true
//! account: bobby

module X {
    struct A { flipper: bool }
    struct B { flipper: bool }

    public fun a(): A { A{flipper: false} }
    public fun flipper_a(x: &A): bool { x.flipper }
    public fun flip_a(x: &mut A) {
        if (x.flipper) x.flipper = false
        else x.flipper = true;
    }

    public fun b(): B { B{flipper: false} }
    public fun flipper_b(x: &B): bool { x.flipper }
    public fun flip_b(x: &mut B) {
        if (x.flipper) x.flipper = false
        else x.flipper = true;
    }
}

//! new-transaction
//! sender: association
script {
use 0x0::Unhosted;
use 0x0::Transaction;
use 0x0::AccountType;

fun main() {
    Transaction::assert(AccountType::is_a<Unhosted::T>(Transaction::sender()), 1);
}
}
// check: EXECUTED

//! new-transaction
//! sender: association
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::apply_for(X::a(), {{bob}});
}
}
// check: ABORTED
// check: 3

//! new-transaction
//! sender: association
script {
use {{default}}::X;
use 0x0::Transaction;
use 0x0::AccountType;

fun main() {
    Transaction::assert(!AccountType::is_a<X::A>(Transaction::sender()), 2);
    Transaction::assert(!AccountType::is_a<X::B>(Transaction::sender()), 3);
    AccountType::register<X::A>();
    AccountType::register<X::B>();
}
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// Publish granting caps under alice and vivian's accounts
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: alice
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::apply_for_granting_capability<X::A>()
}
}
// check: EXECUTED

//! new-transaction
//! sender: alex
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::apply_for_granting_capability<X::A>()
}
}
// check: EXECUTED

//! new-transaction
//! sender: vivian
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::apply_for_granting_capability<X::B>()
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::apply_for_granting_capability<X::B>()
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::apply_for_transition_capability<X::A>({{alice}});
    AccountType::apply_for_transition_capability<X::B>({{vivian}})
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::apply_for_transition_capability<X::A>({{alex}});
}
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// Now apply_for account types to be transitioned to
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: ta
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::apply_for(X::a(), {{alice}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: txa
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    // give the wrong address for granting authority
    AccountType::apply_for(X::a(), {{vivian}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: tb
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::apply_for(X::b(), {{vivian}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: txb
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    // give the wrong address for granting authority
    AccountType::apply_for(X::b(), {{alice}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: bobby
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
fun main() {
    AccountType::apply_for(X::b(), {{vivian}});
}
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// Try transitioning with non-approved TransititionCapabilities
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::transition<X::A>({{ta}});
}
}
// check: ABORTED
// check: 2000


///////////////////////////////////////////////////////////////////////////
// Try granting TransititionCapabilities without an approved GrantingCapability
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: vivian
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::grant_transition_capability<X::B>({{bob}});
}
}
// check: ABORTED
// check: 2001

///////////////////////////////////////////////////////////////////////////
// Try certifying TransititionCapabilities without an approved GrantingCapability
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: association
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::certify_granting_capability<X::A>({{alice}});
    AccountType::certify_granting_capability<X::A>({{alex}});
    AccountType::certify_granting_capability<X::B>({{vivian}});
    AccountType::certify_granting_capability<X::B>({{bob}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::grant_transition_capability<X::A>({{bob}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: vivian
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::grant_transition_capability<X::B>({{bob}});
}
}
// check: EXECUTED

//! new-transaction
//! sender: alex
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;

fun main() {
    AccountType::grant_transition_capability<X::A>({{alice}});
}
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// Now transition the accounts
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
use 0x0::Transaction;

fun main() {
    AccountType::transition<X::A>({{ta}});
    AccountType::transition<X::B>({{tb}});
    Transaction::assert(AccountType::is_a<X::A>({{ta}}), 4);
    Transaction::assert(!AccountType::is_a<X::A>({{tb}}), 5);
    Transaction::assert(AccountType::is_a<X::B>({{tb}}), 6);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
fun main() {
    AccountType::transition<X::B>({{bobby}});
}
}
// check: ABORTED
// check: 2004

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
// Invalid transition of an account.
fun main() {
    AccountType::transition<X::A>({{txa}});
}
}
// check: ABORTED
// check: 2001

///////////////////////////////////////////////////////////////////////////
// Make sure metadata is correct
///////////////////////////////////////////////////////////////////////////

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
use 0x0::Transaction;
// Make sure that the root addresses point to the granting (i.e. root
// authority) address.
fun main() {
    let root_addr = AccountType::root_address<X::A>({{ta}});
    Transaction::assert(root_addr == {{alice}}, 7);
    let root_addr = AccountType::root_address<X::B>({{tb}});
    Transaction::assert(root_addr == {{vivian}}, 8);
}
}
// check: EXECUTED

//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
use 0x0::Transaction;
// Check that the cap predicates are correct
fun main() {
    Transaction::assert(AccountType::has_transition_cap<X::A>({{bob}}), 9);
    Transaction::assert(AccountType::has_transition_cap<X::B>({{bob}}), 10);
    Transaction::assert(AccountType::has_transition_cap<X::A>({{alice}}), 12);
    Transaction::assert(AccountType::has_granting_cap<X::A>({{alice}}), 12);
    Transaction::assert(AccountType::has_granting_cap<X::B>({{vivian}}), 13);
    Transaction::assert(!AccountType::has_transition_cap<X::B>({{alice}}), 14);
    Transaction::assert(!AccountType::has_transition_cap<X::B>({{ta}}), 15);
    Transaction::assert(!AccountType::has_granting_cap<X::A>({{bob}}), 16);
    Transaction::assert(!AccountType::has_granting_cap<X::B>({{alice}}), 17);
}
}
// check: EXECUTED

///////////////////////////////////////////////////////////////////////////
// Make sure that updating the internal account info is updated correctly
///////////////////////////////////////////////////////////////////////////


//! new-transaction
//! sender: bob
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
use 0x0::Transaction;
// normal update
// Make sure that we have interior mutability in the inner value of the
// account type (held in the `account_metadata` section.
fun main() {
    let account_metadata = AccountType::account_metadata<X::A>({{ta}});
    Transaction::assert(!X::flipper_a(&account_metadata), 18);
    X::flip_a(&mut account_metadata);
    Transaction::assert(X::flipper_a(&account_metadata), 19);
    AccountType::update<X::A>({{ta}}, account_metadata);
    let account_metadata = AccountType::account_metadata<X::A>({{ta}});
    Transaction::assert(X::flipper_a(&account_metadata), 20);
}
}
// check: EXECUTED

//! new-transaction
//! sender: alice
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
// Invalid update
// - try to update the account info with a transition capability with the
//   wrong TransitionCapability root address: alice's TransitionCapability
//   points at alex, and the ta account's root address points to alice.
fun main() {
    let account_metadata = AccountType::account_metadata<X::A>({{ta}});
    AccountType::update<X::A>({{ta}}, account_metadata);
}
}
// check: ABORTED
// check: 2001

//! new-transaction
//! sender: vivian
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
// Invalid update
fun main() {
    // This TransitionCapability points at the root address. BUT, it is not
    // yet certified.
    AccountType::apply_for_transition_capability<X::A>({{alice}});
    let account_metadata = AccountType::account_metadata<X::A>({{ta}});
    AccountType::update<X::A>({{ta}}, account_metadata);
}
}
// check: ABORTED
// check: 2005

//! new-transaction
//! sender: alice
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
// Invalid update
// The updating account is not certified of type X::A yet.
fun main() {
    let account_metadata = AccountType::account_metadata<X::A>({{ta}});
    AccountType::update<X::A>({{txa}}, account_metadata);
}
}
// check: ABORTED
// check: 2004

//! new-transaction
//! sender: alice
//! gas-price: 0
script {
use {{default}}::X;
use 0x0::AccountType;
// Invalid account info access
// The account has a published AccountType::<X::A> but is not certified of type X::A yet.
fun main() {
    AccountType::account_metadata<X::A>({{txa}});
}
}
// check: ABORTED
// check: 2004

//! new-transaction
//! sender: association
script {
use {{default}}::X;
use 0x0::AccountType;
// valid revocation
// alice has the right granting cap: GrantingCapability<X::A>
fun main() {
    AccountType::remove_granting_capability<X::B>({{vivian}});
}
}
// check: EXECUTED

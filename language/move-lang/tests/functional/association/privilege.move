//! account: bob, 100000
//! account: alice, 100000

module Privilege {
    struct A {}
    struct B {}
}

//! new-transaction
//! sender: bob
script {
use 0x0::Association;
fun main() {
    Association::assert_sender_is_association();
}
}
// check: ABORTED
// check: 1002

// Make Alice an association account

//! new-transaction
//! sender: alice
script {
use 0x0::Association;
use 0x0::Transaction;
fun main() {
    Association::apply_for_privilege<Association::T>();
    Transaction::assert(!Association::addr_is_association({{alice}}), 0);
}
}

//! new-transaction
//! sender: association
script {
use 0x0::Association;
fun main() {
    Association::grant_association_address({{alice}});
}
}

// Now check privilege flows

//! new-transaction
//! sender: bob
script {
use 0x0::Association;
use {{default}}::Privilege;
// Bob publishes a bunch of privileges
fun main() {
    Association::apply_for_privilege<Association::T>();
    Association::apply_for_privilege<Privilege::A>();
    Association::apply_for_privilege<Privilege::B>();
}
}

//! new-transaction
//! sender: alice
script {
use 0x0::Association;
// Make sure only the root association account can make new association
// accounts.
fun main() {
    Association::grant_association_address({{bob}});
}
}
// check: ABORTED
// check: 1001

//! new-transaction
//! sender: association
script {
use 0x0::Association;
use 0x0::Transaction as T;
use {{default}}::Privilege;
// Make sure certification of privileges is dependent on certification as
// an association account.
fun main() {
    // other privilege is dependent on certification as an association
    // account.
    Association::grant_privilege<Privilege::A>({{bob}});
    T::assert(!Association::has_privilege<Privilege::A>({{bob}}), 1);
    T::assert(!Association::has_privilege<Privilege::B>({{bob}}), 2);

    // Once certified, other privilege becomes live. But privilege also dependent on
    // certification of itself too.
    Association::grant_association_address({{bob}});
    T::assert(Association::has_privilege<Privilege::A>({{bob}}), 3);
    T::assert(!Association::has_privilege<Privilege::B>({{bob}}), 4);

    // Removing from the association decertifies other privileges.
    Association::remove_privilege<Association::T>({{bob}});
    T::assert(!Association::has_privilege<Privilege::A>({{bob}}), 5);
}
}

//! new-transaction
//! sender: association
script {
use 0x0::Association;
// Resource no longer exists
fun main() {
    Association::grant_association_address({{bob}});
}
}
// check: ABORTED
// check: 1003

//! new-transaction
//! sender: alice
script {
use 0x0::Association;
use {{default}}::Privilege;
// Normal association accounts can't remove privileges
fun main() {
    Association::remove_privilege<Privilege::A>({{bob}});
}
}
// check: ABORTED
// check: 1001

//! new-transaction
//! sender: association
script {
use 0x0::Association;
use {{default}}::Privilege;
// Root account can remove privileges. These can either be certified or
// uncertified.
fun main() {
    Association::remove_privilege<Privilege::A>({{bob}});
    Association::remove_privilege<Privilege::B>({{bob}});
    Association::remove_privilege<Association::T>({{alice}});
}
}

//! new-transaction
//! sender: alice
script {
use 0x0::Association;
// Make sure alice is no longer an association account
fun main() {
    Association::assert_sender_is_association();
}
}
// check: ABORTED
// check: 1002

//! new-transaction
script {
use 0x0::Association;
// Make sure alice is no longer an association account
fun main() {
    Association::initialize();
}
}
// check: ABORTED
// check: 1000

//! new-transaction
//! sender: association
script {
use 0x0::Association;
// Make sure alice is no longer an association account
fun main() {
    Association::remove_privilege<Association::T>({{alice}});
}
}
// check: ABORTED
// check: 1004

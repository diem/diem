// Implements logic for registering addresses as association addresses, and
// determining if the sending account is an association account.
// Errors:
// 1000 -> INVALID_GENESIS_ADDR
// 1001 -> INSUFFICIENT_PRIVILEGES
// 1002 -> NOT_AN_ASSOCIATION_ACCOUNT
// 1003 -> ACCOUNT_DOES_NOT_HAVE_PRIVILEGE
// 1004 -> ACCOUNT_DOES_NOT_HAVE_PRIVILEGE_RESOURCE
address 0x0:

module Association {
    use 0x0::Transaction;
    use 0x0::Sender;

    // The root account privilege. This is created at genesis and has
    // special privileges (e.g. removing an account as an association
    // account). It cannot be removed.
    resource struct Root { }

    // There are certain association capabilities that are more
    // privileged than other association operations. This resource with the
    // type representing that privilege is published under the privileged
    // account.
    resource struct PrivilegedCapability<Privilege> { is_certified: bool }

    // A type tag to mark that this account is an association account.
    // It cannot be used for more specific/privileged operations.
    struct T { }

    // Initialization is called in genesis. It publishes the root resource
    // under the root_address() address, marks it as a normal
    // association account.
    public fun initialize(sender: &Sender::T) {
        Transaction::assert(Sender::address_(sender) == root_address(), 1000);
        Sender::move_to(sender);
        move_to_sender(Root{ });
        Sender::move_to(sender);
        move_to_sender(PrivilegedCapability<T>{ is_certified: true });
    }

    // Publish a specific privilege under the sending account.
    public fun apply_for_privilege<Privilege>(sender: &Sender::T) {
        Sender::move_to(sender);
        move_to_sender(PrivilegedCapability<Privilege>{ is_certified: false });
    }

    // Certify the privileged capability published under for_addr.
    public fun grant_privilege<Privilege>(for_addr: address, sender: &Sender::T)
    acquires PrivilegedCapability {
        assert_sender_is_root(sender);
        Transaction::assert(exists<PrivilegedCapability<Privilege>>(for_addr), 1003);
        borrow_global_mut<PrivilegedCapability<Privilege>>(for_addr).is_certified = true;
    }

    // Return whether the `addr` has the specified `Privilege`.
    public fun has_privilege<Privilege>(addr: address): bool
    acquires PrivilegedCapability {
        addr_is_association(addr) &&
        exists<PrivilegedCapability<Privilege>>(addr) &&
        borrow_global<PrivilegedCapability<Privilege>>(addr).is_certified
    }

    // Remove the `Privilege` from the address at `addr`. The sender must
    // be the root association account. The `Privilege` need not be
    // certified.
    public fun remove_privilege<Privilege>(addr: address, sender: &Sender::T)
    acquires PrivilegedCapability {
        assert_sender_is_root(sender);
        Transaction::assert(exists<PrivilegedCapability<Privilege>>(addr), 1004);
        PrivilegedCapability<Privilege>{ is_certified: _ } = move_from<PrivilegedCapability<Privilege>>(addr);
    }

    // Publishes an Association::PrivilegedCapability<T> under the sending
    // account.
    public fun apply_for_association(sender: &Sender::T) {
        apply_for_privilege<T>(sender)
    }

    // Certifies the Association::PrivilegedCapability<T> resource that is
    // published under `addr`.
    public fun grant_association_address(addr: address, sender: &Sender::T)
    acquires PrivilegedCapability {
        grant_privilege<T>(addr, sender)
    }

    // Assert that the sender is an association account.
    public fun assert_sender_is_association(sender: &Sender::T)
    acquires PrivilegedCapability {
        assert_addr_is_association(Sender::address_(sender))
    }

    // Assert that the sender is the root association account.
    public fun assert_sender_is_root(sender: &Sender::T) {
        Transaction::assert(exists<Root>(Sender::address_(sender)), 1001);
    }

    // Return whether the account at `addr` is an association account.
    public fun addr_is_association(addr: address): bool
    acquires PrivilegedCapability {
        exists<PrivilegedCapability<T>>(addr) &&
            borrow_global<PrivilegedCapability<T>>(addr).is_certified
    }

    // The address at which the root account will be published.
    public fun root_address(): address {
        0xA550C18
    }

    // Assert that `addr` is an association account.
    fun assert_addr_is_association(addr: address)
    acquires PrivilegedCapability {
        Transaction::assert(addr_is_association(addr), 1002);
    }
}

address 0x2 {
// a distributed key-value map is used to store name entry (name, address, expiration_date)
// key is the name(:vector<u8>), stored in a sorted linked list
// value is a struct 'Expiration', contains the expiration date of the name
// the account address of each list node is actually the address bound to the key(name)
module NameService {
    use 0x2::SortedLinkedList::{Self, EntryHandle};
    use DiemFramework::DiemBlock;
    use Std::Signer;
    use Std::Vector;

    //TODO use constants when Move support constants, '5' is used for example
    public fun EXPIRE_AFTER() : u64{5}
    const NAMESERVICE_ADDR: address = @0x2;

    struct Expiration has key {
        expire_on_block_height: vector<u64>
    }

    public fun entry_handle(addr: address, index: u64): EntryHandle {
        SortedLinkedList::entry_handle(addr, index)
    }

    public fun initialize(account: &signer) {
        let sender = Signer::address_of(account);
        assert(sender == NAMESERVICE_ADDR, 8000);

        SortedLinkedList::create_new_list<vector<u8>>(account, Vector::empty());
        move_to<Expiration>(account, Expiration { expire_on_block_height: Vector::singleton(0u64)});
    }

    fun add_expirtation(account: &signer) acquires Expiration {
        let sender = Signer::address_of(account);
        let current_block = DiemBlock::get_current_block_height();
        if (!exists<Expiration>(sender)) {
            move_to<Expiration>(account, Expiration {expire_on_block_height: Vector::singleton(current_block + EXPIRE_AFTER())});
        } else {
            let expire_vector_mut = &mut borrow_global_mut<Expiration>(sender).expire_on_block_height;
            Vector::push_back<u64>(expire_vector_mut, current_block + EXPIRE_AFTER());
        };
    }

    public fun add_name(account: &signer, name: vector<u8>, prev_entry: EntryHandle) acquires Expiration {
        SortedLinkedList::insert_node(account, name, prev_entry);
        Self::add_expirtation(account);
    }

    public fun get_name_for(entry: EntryHandle): vector<u8> {
        SortedLinkedList::get_data<vector<u8>>(entry)
    }

    fun remove_expiration(entry: EntryHandle) acquires Expiration {
        let account_address = SortedLinkedList::get_addr(copy entry);
        let index = SortedLinkedList::get_index(entry);
        let expire_vector_mut = &mut borrow_global_mut<Expiration>(account_address).expire_on_block_height;
        Vector::remove<u64>(expire_vector_mut, index);
        if (Vector::is_empty<u64>(expire_vector_mut)) {
            let Expiration { expire_on_block_height } = move_from<Expiration>(account_address);
            Vector::destroy_empty(expire_on_block_height);
        }
    }
    public fun remove_entry_by_entry_owner(account: &signer, entry: EntryHandle) acquires Expiration {
        SortedLinkedList::remove_node_by_node_owner<vector<u8>>(account, copy entry);
        Self::remove_expiration(entry);
    }

    public fun remove_entry_by_service_owner(account: &signer, entry: EntryHandle) acquires Expiration {
        SortedLinkedList::remove_node_by_list_owner<vector<u8>>(account, copy entry);
        Self::remove_expiration(entry);
    }

    public fun find_position_and_insert(account: &signer, name: vector<u8>, head: EntryHandle): bool acquires Expiration {
        if (SortedLinkedList::find_position_and_insert<vector<u8>>(account, name, head)) {
            Self::add_expirtation(account);
            return true
        } else {
            return false
        }
    }

    public fun is_head_entry(entry: EntryHandle): bool {
        SortedLinkedList::is_head_node<vector<u8>>(&entry)
    }

    public fun expire_on_block_height(entry: EntryHandle): u64 acquires Expiration {
        let addr = SortedLinkedList::get_addr(copy entry);
        let index = SortedLinkedList::get_index(entry);
        let expire_vector = *&borrow_global<Expiration>(addr).expire_on_block_height;
        *Vector::borrow<u64>(&expire_vector, index)
    }

    public fun is_expired(entry: EntryHandle): bool acquires Expiration {
        let current_block_height = DiemBlock::get_current_block_height();
        current_block_height > expire_on_block_height(entry)
    }
}
}

/*
//! new-transaction
//! sender: nameservice
//initialize the nameservice list
script {
use {{nameservice}}::NameService;
fun main(account: signer) {
    NameService::initialize(account);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
//adding a new name to NameService's list _@nameservice -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: signer) {
    let head = NameService::entry_handle({{nameservice}}, 0);
    NameService::find_position_and_insert(account, b"alice", head);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
//adding a new name to NameService's list _@nameservice -> b"bob"@bob -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: signer) {
    let head = NameService::entry_handle({{nameservice}}, 0);
    NameService::find_position_and_insert(account, b"bob", head);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: carol
//adding a new name to NameService's list _@nameservice -> b"bob"@bob -> b"alice"@alice -> b"carol"@carol
script {
use {{nameservice}}::NameService;
fun main(account: signer) {
    let head = NameService::entry_handle({{nameservice}}, 0);
    NameService::find_position_and_insert(account, b"carol", head);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: david
//ensure the entry under {{alice}} holds the name b"alice"
script {
use {{nameservice}}::NameService;
fun main() {
    let entry = NameService::entry_handle({{alice}}, 0);
    let name = NameService::get_name_for(entry);
    assert(name == b"alice", 26);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: carol
//removes her entry _@nameservice -> b"bob"@bob -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: signer) {
    let entry = NameService::entry_handle({{carol}}, 0);
    NameService::remove_entry_by_entry_owner(account, entry);
}
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: nameservice
//removes bob's entry _@nameservice -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: signer) {
    let entry = NameService::entry_handle({{bob}}, 0);
    assert(NameService::is_expired(copy entry), 27);
    NameService::remove_entry_by_service_owner(account, entry);
}
}
// check: ABORTED


//! block-prologue
//! proposer: vivian
//! block-time: 1001

//! block-prologue
//! proposer: vivian
//! block-time: 1002

//! block-prologue
//! proposer: vivian
//! block-time: 1003

//! block-prologue
//! proposer: vivian
//! block-time: 1004

//! block-prologue
//! proposer: vivian
//! block-time: 1005

//! block-prologue
//! proposer: vivian
//! block-time: 1006

//! new-transaction
//! sender: nameservice
//removes her entry _@nameservice -> b"alice"@alice
script {
use {{nameservice}}::NameService;
fun main(account: signer) {
    let entry = NameService::entry_handle({{bob}}, 0);
    assert(NameService::is_expired(copy entry), 27);
    NameService::remove_entry_by_service_owner(account, entry);
}
}
// check: "Keep(EXECUTED)"
*/

//! account: sys
//! account: nftservice
//! account: alice
//! account: bob

//! new-transaction
//! sender: sys
module SortedLinkedList {
    use 0x1::Compare;
    use 0x1::LCS;
    use 0x1::Signer;

    resource struct Node<T> {
        prev: address, //account address where the previous node is stored (head if no previous node exists)
        next: address, //account address where the next node is stored (head if no next node exists)
        head: address, //account address where current list's head is stored -- whoever stores head is the owner of the whole list
        key: T
    }

    public fun node_exists<T: copyable>(node_address: address): bool {
        exists<Node<T>>(node_address)
    }

    public fun get_key_of_node<T: copyable>(node_address: address): T acquires Node {
        assert(exists<Node<T>>(node_address), 1);

        let node = borrow_global<Node<T>>(node_address);
        *&node.key
    }

    //checks whether this address is the head of a list -- fails if there is no node here
    public fun is_head_node<T: copyable>(current_node_address: address): bool acquires Node {
		//check that a node exists
		assert(exists<Node<T>>(current_node_address), 2);

        //find the head node
		let current_node = borrow_global<Node<T>>(current_node_address);
        let head_node_address = current_node.head;

        //check if this is the head node
        head_node_address == current_node_address
    }

    //creates a new list whose head is at txn_sender (is owned by the caller)
    public fun create_new_list<T: copyable>(account: &signer, key: T) {
        let sender = Signer::address_of(account);

        //make sure no node/list is already stored in this account
        assert(!exists<Node<T>>(sender), 3);

        let head = Self::Node<T> {
            prev: sender,
            next: sender,
            head: sender,
            key: key
        };
        move_to<Node<T>>(account, head);
    }

    //adds a node that is stored in txn_sender's account and whose location in the list is right after prev_node_address
    public fun add_node<T: copyable>(account: &signer, key: T, prev_node_address: address) acquires Node {
        let sender_address = Signer::address_of(account);

        //make sure no node is already stored in this account
        assert(!exists<Node<T>>(sender_address), 4);

        //make sure a node exists in prev_node_address
        assert(exists<Node<T>>(prev_node_address), 5);

        //get a reference to prev_node and find the address and reference to next_node, head
        let prev_node = borrow_global<Node<T>>(prev_node_address);
        let next_node_address = prev_node.next;
        let next_node = borrow_global<Node<T>>(next_node_address);
        let head_address = next_node.head;

        //see if either prev or next are the head and get their keys
        let prev_key = *&prev_node.key;
        let next_key = *&next_node.key;
        let key_lcs_bytes = LCS::to_bytes(&key);
        let cmp_with_prev = Compare::cmp_lcs_bytes(&key_lcs_bytes, &LCS::to_bytes(&prev_key));
        let cmp_with_next = Compare::cmp_lcs_bytes(&key_lcs_bytes, &LCS::to_bytes(&next_key));

        let prev_is_head = Self::is_head_node<T>(prev_node_address);
        let next_is_head = Self::is_head_node<T>(next_node_address);

        //check the order -- the list must be sorted
        assert(prev_is_head || cmp_with_prev == 2u8, 6); // prev_is_head || key > prev_key
        assert(next_is_head || cmp_with_next == 1u8, 7); // next_is_head || key < next_key

        //create the new node
        let current_node = Node<T> {
            prev: prev_node_address,
            next: next_node_address,
            head: head_address,
            key: key
        };
        move_to<Node<T>>(account, current_node);

        //fix the pointers at prev
        let prev_node_mut = borrow_global_mut<Node<T>>(prev_node_address);
        prev_node_mut.next = sender_address;

        //fix the pointers at next
        let next_node_mut = borrow_global_mut<Node<T>>(next_node_address);
        next_node_mut.prev = sender_address;
    }

    //private function used for removing a non-head node -- does not check permissions
    fun remove_node<T: copyable>(node_address: address) acquires Node {
        //make sure the node exists
        assert(exists<Node<T>>(node_address), 8);

        //find prev and next
        let current_node = borrow_global<Node<T>>(node_address);
        let next_node_address = current_node.next;
        let prev_node_address = current_node.prev;

        //update next
        let next_node_mut = borrow_global_mut<Node<T>>(next_node_address);
        next_node_mut.prev = prev_node_address;

        //update prev
        let prev_node_mut = borrow_global_mut<Node<T>>(prev_node_address);
        prev_node_mut.next = next_node_address;

        //destroy the current node
        let Node<T> { prev: _, next: _, head: _, key: _ } = move_from<Node<T>>(node_address);
    }

    public fun remove_node_by_list_owner<T: copyable>(account: &signer, node_address: address) acquires Node {
        //make sure the node exists
        assert(exists<Node<T>>(node_address), 9);

        //make sure it is not a head node
        assert(!Self::is_head_node<T>(node_address), 10);

        //make sure the caller owns the list
        let node = borrow_global<Node<T>>(node_address);
        let list_owner = node.head;
        assert(list_owner == Signer::address_of(account), 11);

        //remove it
        Self::remove_node<T>(node_address);
    }

    //removes the current non-head node -- fails if the passed node is the head of a list
    public fun remove_node_by_node_owner<T: copyable>(account: &signer) acquires Node {
        let sender_address = Signer::address_of(account);

        //make sure a node exists
        assert(exists<Node<T>>(sender_address), 12);

        //make sure it is not a head node (heads can be removed using remove_list)
        assert(!Self::is_head_node<T>(sender_address), 13);

        //remove it
        Self::remove_node<T>(sender_address);
    }

    //can only called by the list owner (head) -- removes the list if it is empty
    //fails if it is non-empty or if no list is owned by the caller
    public fun remove_list<T: copyable>(account: &signer) acquires Node {
        let sender_address = Signer::address_of(account);

        //fail if the caller does not own a list
        assert(Self::is_head_node<T>(sender_address), 14);

        assert(exists<Node<T>>(sender_address), 15);
        let current_node = borrow_global<Node<T>>(sender_address);

        //check that the list is empty
        let next_node_address = current_node.next;
        let prev_node_address = current_node.prev;
        assert(next_node_address == sender_address, 16);
        assert(prev_node_address == sender_address, 17);

        //destroy the Node
        let Node<T> { prev: _, next: _, head: _, key: _ } = move_from<Node<T>>(sender_address);
    }

    public fun find<T: copyable>(key: T, head_address: address): (bool, address) acquires Node {
        assert(Self::is_head_node<T>(head_address), 18);

        let key_lcs_bytes = LCS::to_bytes(&key);
        let head_node = borrow_global<Node<T>>(head_address);
        let next_node_address = head_node.next;
        while (next_node_address != head_address) {
            let next_node = borrow_global<Node<T>>(next_node_address);
            let next_node_key = *&next_node.key;
            let next_key_lcs_bytes = LCS::to_bytes(&next_node_key);
            let cmp = Compare::cmp_lcs_bytes(&next_key_lcs_bytes, &key_lcs_bytes);

            if (cmp == 0u8) { // next_key == key
                return (true, next_node_address)
            } else if (cmp == 1u8) { // next_key < key, continue
                next_node_address = *&next_node.next;
            } else { // next_key > key, nothing found
                let prev_node_address = *&next_node.prev;
                return (false, prev_node_address)
            }
        };
        return (false, *&head_node.prev)
    }

    public fun empty_node<T: copyable>(account: &signer, key: T) {
        let sender = Signer::address_of(account);

        //make sure no node/list is already stored in this account
        assert(!exists<Node<T>>(sender), 19);

        let empty = Self::Node<T> {
            prev: sender,
            next: sender,
            head: sender,
            key: key
        };
        move_to<Node<T>>(account, empty);
    }

    public fun move_node_to<T: copyable>(account: &signer, receiver: address) acquires Node {
        let sender_address = Signer::address_of(account);
        //make sure the node exists
        assert(exists<Node<T>>(sender_address), 20);
        assert(exists<Node<T>>(receiver), 21);  //empty node

        //find prev and next
        let current_node = borrow_global<Node<T>>(sender_address);
        let next_node_address = current_node.next;
        let prev_node_address = current_node.prev;

        //update next
        let next_node_mut = borrow_global_mut<Node<T>>(next_node_address);
        next_node_mut.prev = receiver;

        //update prev
        let prev_node_mut = borrow_global_mut<Node<T>>(prev_node_address);
        prev_node_mut.next = receiver;

        let Node<T> { prev, next, head, key } = move_from<Node<T>>(sender_address);
        let receiver_node_mut = borrow_global_mut<Node<T>>(receiver);
        receiver_node_mut.prev = prev;
        receiver_node_mut.next = next;
        receiver_node_mut.head = head;
        receiver_node_mut.key = key;

    }

}

//! new-transaction
//! sender: nftservice

// a distributed key-value map is used to store entry (token_id, address, NonFungibleToken)
// key is the token_id(:vector<u8>), stored in a sorted linked list
// value is a struct 'NonFungibleToken', contains the non fungible token
// the account address of each list node is actually the owner of the token
module NonFungibleToken {
    use {{sys}}::SortedLinkedList;
    use 0x1::Option::{Self, Option};
    use 0x1::Signer;
    use 0x1::Vector;

    resource struct LimitedMeta {
        limited: bool,
        total: u64,
    }

    resource struct NonFungibleToken<Token> {
        token: Option<Token>
    }

    resource struct TokenLock<Token> {
    }

    fun lock<Token>(account: &signer) {
        move_to<TokenLock<Token>>(account, TokenLock<Token>{});
    }

    fun unlock<Token>(account: &signer) acquires TokenLock {
        let sender = Signer::address_of(account);
        let TokenLock<Token> {} = move_from<TokenLock<Token>>(sender);
    }

    public fun initialize<Token>(account: &signer, limited: bool, total: u64) {
        let sender = Signer::address_of(account);
        assert(sender == {{nftservice}}, 8000);

        let limited_meta = LimitedMeta {
            limited: limited,
            total: total,
        };
        move_to<LimitedMeta>(account, limited_meta);
        SortedLinkedList::create_new_list<vector<u8>>(account, Vector::empty());
    }

    public fun preemptive<Token>(account: &signer, nft_service_address: address, token_id: vector<u8>, token: Token):Option<Token> {
        let (exist, location) = Self::find(copy token_id, nft_service_address);
        if (exist) return Option::some(token);

        SortedLinkedList::add_node<vector<u8>>(account, token_id, location);
        move_to<NonFungibleToken<Token>>(account, NonFungibleToken<Token>{token: Option::some(token)});
        Option::none() //preemptive success
    }

    public fun accept_token<Token>(account: &signer) {
        let sender = Signer::address_of(account);
        assert(!exists<NonFungibleToken<Token>>(sender), 8001);
        SortedLinkedList::empty_node<vector<u8>>(account, Vector::empty());
        move_to<NonFungibleToken<Token>>(account, NonFungibleToken<Token>{token: Option::none()});
    }

    public fun safe_transfer<Token: copyable>(account: &signer, _nft_service_address: address, token_id: vector<u8>, receiver: address) acquires NonFungibleToken {
        let sender = Signer::address_of(account);
        assert(exists<NonFungibleToken<Token>>(receiver), 8002);
        assert(Option::is_none(&borrow_global<NonFungibleToken<Token>>(receiver).token), 8005);
        assert(Self::get_token_id(sender) == token_id, 8003);
        assert(!exists<TokenLock<Token>>(sender), 8004);

        SortedLinkedList::move_node_to<vector<u8>>(account, receiver);
        let NonFungibleToken<Token>{ token } = move_from<NonFungibleToken<Token>>(sender);
        let receiver_token_ref_mut = borrow_global_mut<NonFungibleToken<Token>>(receiver);
        receiver_token_ref_mut.token = token;
    }

    public fun get_token_id(addr: address): vector<u8> {
        SortedLinkedList::get_key_of_node<vector<u8>>(addr)
    }

    public fun find(token_id: vector<u8>, head_address: address): (bool, address) {
        SortedLinkedList::find<vector<u8>>(token_id, head_address)
    }

    public fun get_nft<Token>(account: &signer): NonFungibleToken<Token> acquires NonFungibleToken {
        let sender = Signer::address_of(account);
        assert(exists<NonFungibleToken<Token>>(sender), 8006);
        assert(!exists<TokenLock<Token>>(sender), 8007);
        Self::lock<Token>(account);
        move_from<NonFungibleToken<Token>>(sender)
    }

    public fun put_nft<Token>(account: &signer, nft: NonFungibleToken<Token>) acquires TokenLock {
        let sender = Signer::address_of(account);
        assert(exists<TokenLock<Token>>(sender), 8008);
        Self::unlock<Token>(account);
        move_to<NonFungibleToken<Token>>(account, nft)
    }
}

//! new-transaction
//! sender: nftservice
module TestNft {
    struct TestNft {}
    public fun new_test_nft(): TestNft {
        TestNft{}
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
// sample for moving Nft into another resource
module MoveNft {
    use {{nftservice}}::NonFungibleToken::{Self, NonFungibleToken};
    use {{nftservice}}::TestNft::TestNft;
    use 0x1::Signer;

    resource struct MoveNft {
        nft: NonFungibleToken<TestNft>
    }

    public fun move_nft(account: &signer) {
        let nft = NonFungibleToken::get_nft<TestNft>(account);
        move_to<MoveNft>(account, MoveNft{ nft });
    }

    public fun move_back_nft(account: &signer) acquires MoveNft {
        let sender = Signer::address_of(account);
        let MoveNft { nft } = move_from<MoveNft>(sender);
        NonFungibleToken::put_nft<TestNft>(account, nft);
    }
}
// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: nftservice
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::TestNft;
fun main(account: &signer) {
    NonFungibleToken::initialize<TestNft>(account, false, 0);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::{Self, TestNft};
use 0x1::Hash;
fun main(account: &signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    let token = TestNft::new_test_nft();
    NonFungibleToken::preemptive<TestNft>(account, {{nftservice}}, token_id, token);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{alice}}::MoveNft;
fun main(account: &signer) {
    MoveNft::move_nft(account);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: bob
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::TestNft;
fun main(account: &signer) {
    NonFungibleToken::accept_token<TestNft>(account);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::TestNft;
use 0x1::Hash;
fun main(account: &signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    NonFungibleToken::safe_transfer<TestNft>(account, {{nftservice}}, token_id, {{bob}});
}
}

// check: ABORTED

//! new-transaction
//! sender: alice
script {
use {{alice}}::MoveNft;
fun main(account: &signer) {
    MoveNft::move_back_nft(account);
}
}

// check: "Keep(EXECUTED)"

//! new-transaction
//! sender: alice
script {
use {{nftservice}}::NonFungibleToken;
use {{nftservice}}::TestNft::TestNft;
use 0x1::Hash;
fun main(account: &signer) {
    let input = b"input";
    let token_id = Hash::sha2_256(input);
    NonFungibleToken::safe_transfer<TestNft>(account, {{nftservice}}, token_id, {{bob}});
}
}

// check: "Keep(EXECUTED)"

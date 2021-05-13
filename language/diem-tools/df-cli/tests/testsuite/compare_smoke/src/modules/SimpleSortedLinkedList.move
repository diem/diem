address 0x2 {
module SimpleSortedLinkedList {
    use Std::Compare;
    use Std::BCS;
    use Std::Signer;

    struct Node<T> has key {
        prev: address, //account address where the previous node is stored (head if no previous node exists)
        next: address, //account address where the next node is stored (head if no next node exists)
        head: address, //account address where current list's head is stored -- whoever stores head is the owner of the whole list
        key: T
    }

    public fun node_exists<T: copy + drop + store>(node_address: address): bool {
        exists<Node<T>>(node_address)
    }

    public fun get_key_of_node<T: copy + drop + store>(node_address: address): T acquires Node {
        assert(exists<Node<T>>(node_address), 1);

        let node = borrow_global<Node<T>>(node_address);
        *&node.key
    }

    //checks whether this address is the head of a list -- fails if there is no node here
    public fun is_head_node<T: copy + drop + store>(current_node_address: address): bool acquires Node {
		//check that a node exists
		assert(exists<Node<T>>(current_node_address), 2);

        //find the head node
		let current_node = borrow_global<Node<T>>(current_node_address);
        let head_node_address = current_node.head;

        //check if this is the head node
        head_node_address == current_node_address
    }

    //creates a new list whose head is at txn_sender (is owned by the caller)
    public fun create_new_list<T: copy + drop + store>(account: &signer, key: T) {
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
    public fun add_node<T: copy + drop + store>(account: &signer, key: T, prev_node_address: address) acquires Node {
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
        let key_bcs_bytes = BCS::to_bytes(&key);
        let cmp_with_prev = Compare::cmp_bcs_bytes(&key_bcs_bytes, &BCS::to_bytes(&prev_key));
        let cmp_with_next = Compare::cmp_bcs_bytes(&key_bcs_bytes, &BCS::to_bytes(&next_key));

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
    fun remove_node<T: copy + drop + store>(node_address: address) acquires Node {
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

    public fun remove_node_by_list_owner<T: copy + drop + store>(account: &signer, node_address: address) acquires Node {
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
    public fun remove_node_by_node_owner<T: copy + drop + store>(account: &signer) acquires Node {
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
    public fun remove_list<T: copy + drop + store>(account: &signer) acquires Node {
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

    public fun find<T: copy + drop + store>(key: T, head_address: address): (bool, address) acquires Node {
        assert(Self::is_head_node<T>(head_address), 18);

        let key_bcs_bytes = BCS::to_bytes(&key);
        let head_node = borrow_global<Node<T>>(head_address);
        let next_node_address = head_node.next;
        while (next_node_address != head_address) {
            let next_node = borrow_global<Node<T>>(next_node_address);
            let next_node_key = *&next_node.key;
            let next_key_bcs_bytes = BCS::to_bytes(&next_node_key);
            let cmp = Compare::cmp_bcs_bytes(&next_key_bcs_bytes, &key_bcs_bytes);

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

    public fun empty_node<T: copy + drop + store>(account: &signer, key: T) {
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

    public fun move_node_to<T: copy + drop + store>(account: &signer, receiver: address) acquires Node {
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
}

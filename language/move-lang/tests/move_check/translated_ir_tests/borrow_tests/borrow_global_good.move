module A {
    use 0x0::Transaction;
    resource struct T {v: u64}
    resource struct U {v: u64}

    public fun A0() acquires T {
        let sender = Transaction::sender();
        let t_ref = borrow_global_mut<T>(sender);
        move t_ref;
        borrow_global_mut<T>(sender);
    }

    public fun A1() acquires T, U {
        let sender = Transaction::sender();
        let t_ref = borrow_global_mut<T>(sender);
        let u_ref = borrow_global_mut<U>(sender);
        t_ref;
        u_ref;
    }

    public fun A2(b: bool) acquires T {
        let sender = Transaction::sender();
        let t_ref = if (b) borrow_global_mut<T>(sender) else borrow_global_mut<T>(sender);
        t_ref;
    }

    public fun A3(b: bool) acquires T {
        let sender = Transaction::sender();
        if (b) {
            borrow_global_mut<T>(sender);
        }
    }

    public fun A4() acquires T {
        let sender = Transaction::sender();
        let x = move_from<T>(sender);
        borrow_global_mut<T>(sender);
        move_to_sender<T>(x);
    }

    public fun A5() acquires T {
        borrow_global<T>(Transaction::sender());
    }
}

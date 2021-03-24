
module M {
    use 0x1::Event::{EventHandle, emit_event, new_event_handle};
    use 0x1::Signer::address_of;

    struct Box<T> has copy, drop, store { x: T }
    struct Box3<T> has copy, drop, store { x: Box<Box<T>> }
    struct Box7<T> has copy, drop, store { x: Box3<Box3<T>> }
    struct Box15<T> has copy, drop, store { x: Box7<Box7<T>> }
    struct Box31<T> has copy, drop, store { x: Box15<Box15<T>> }
    struct Box63<T> has copy, drop, store { x: Box31<Box31<T>> }
    struct Box127<T> has copy, drop, store { x: Box63<Box63<T>> }
    struct Box255<T> has copy, drop, store { x: Box127<Box127<T>> }

    struct MyEvent<T: copy + drop + store> has key {
        e: EventHandle<T>
    }

    fun box3<T>(x: T): Box3<T> {
        Box3 { x: Box { x: Box { x } } }
    }

    fun box7<T>(x: T): Box7<T> {
        Box7 { x: box3(box3(x)) }
    }

    fun box15<T>(x: T): Box15<T> {
        Box15 { x: box7(box7(x)) }
    }

    fun box31<T>(x: T): Box31<T> {
        Box31 { x: box15(box15(x)) }
    }

    fun box63<T>(x: T): Box63<T> {
        Box63 { x: box31(box31(x)) }
    }

    fun box127<T>(x: T): Box127<T> {
        Box127 { x: box63(box63(x)) }
    }

    fun box255<T>(x: T): Box255<T> {
        Box255 { x: box127(box127(x)) }
    }

    fun maybe_init_event<T: copy + drop + store>(s: &signer) {
        if (exists<MyEvent<T>>(address_of(s))) return;

        move_to(s, MyEvent { e: new_event_handle<T>(s)})
    }

    public fun event_128(s: &signer) acquires MyEvent {
        maybe_init_event<Box127<bool>>(s);

        emit_event(&mut borrow_global_mut<MyEvent<Box127<bool>>>(address_of(s)).e, box127(true))
    }

    public fun event_256(s: &signer) acquires MyEvent {
        maybe_init_event<Box255<bool>>(s);

        emit_event(&mut borrow_global_mut<MyEvent<Box255<bool>>>(address_of(s)).e, box255(true))
    }

    public fun event_257(s: &signer) acquires MyEvent {
        maybe_init_event<Box<Box255<bool>>>(s);

        emit_event(
            &mut borrow_global_mut<MyEvent<Box<Box255<bool>>>>(address_of(s)).e,
            Box { x: box255(true) }
        )
    }
}
// check: EXECUTED


//! new-transaction
script {
    use {{default}}::M;

    fun main(s: signer) {
    let s = &s;
        M::event_128(s);
    }
}
// check: EXECUTED


//! new-transaction
script {
    use {{default}}::M;

    fun main(s: signer) {
    let s = &s;
        M::event_256(s);
    }
}
// check: EXECUTED


//! new-transaction
script {
    use {{default}}::M;

    fun main(s: signer) {
    let s = &s;
        M::event_257(s);
    }
}
// check: "ABORTED { code: 0, location: 00000000000000000000000000000001::Event }"

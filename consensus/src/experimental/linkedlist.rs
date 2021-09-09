// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

// modified from https://rust-unofficial.github.io/too-many-lists/fourth-final.html (MIT License)

// maybe later we can move this to /common
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

pub struct List<T> {
    pub head: Link<T>,
    pub tail: Link<T>,
}

pub type Link<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct Node<T> {
    pub elem: Option<T>,
    pub next: Link<T>,
    pub prev: Link<T>,
}

impl<T> Node<T> {
    pub fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Node {
            elem: Some(elem),
            prev: None,
            next: None,
        }))
    }

    pub fn next(&self) -> Link<T> {
        self.next.as_ref().cloned()
    }

    pub fn prev(&self) -> Link<T> {
        self.prev.as_ref().cloned()
    }

    pub fn elem(&self) -> &T {
        self.elem.as_ref().unwrap()
    }

    pub fn elem_mut(&mut self) -> &mut T {
        self.elem.as_mut().unwrap()
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let new_head = Node::new(elem);
        match self.head.take() {
            Some(old_head) => {
                (*old_head).borrow_mut().prev = Some(new_head.clone());
                (*new_head).borrow_mut().next = Some(old_head);
                self.head = Some(new_head);
            }
            None => {
                self.tail = Some(new_head.clone());
                self.head = Some(new_head);
            }
        }
    }

    pub fn push_back(&mut self, elem: T) {
        let new_tail = Node::new(elem);
        match self.tail.take() {
            Some(old_tail) => {
                (*old_tail).borrow_mut().next = Some(new_tail.clone());
                (*new_tail).borrow_mut().prev = Some(old_tail);
                self.tail = Some(new_tail);
            }
            None => {
                self.head = Some(new_tail.clone());
                self.tail = Some(new_tail);
            }
        }
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            match (*old_tail).borrow_mut().prev.take() {
                Some(new_tail) => {
                    (*new_tail).borrow_mut().next.take();
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head.take();
                }
            }
            Rc::try_unwrap(old_tail)
                .ok()
                .unwrap()
                .into_inner()
                .elem
                .unwrap()
        })
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            match (*old_head).borrow_mut().next.take() {
                Some(new_head) => {
                    (*new_head).borrow_mut().prev.take();
                    self.head = Some(new_head);
                }
                None => {
                    self.tail.take();
                }
            }
            Rc::try_unwrap(old_head)
                .ok()
                .unwrap()
                .into_inner()
                .elem
                .unwrap()
        })
    }

    pub fn peek_front(&self) -> Option<Ref<T>> {
        self.head
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| node.elem.as_ref().unwrap()))
    }

    pub fn peek_back(&self) -> Option<Ref<T>> {
        self.tail
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| node.elem.as_ref().unwrap()))
    }

    pub fn peek_back_mut(&mut self) -> Option<RefMut<T>> {
        self.tail
            .as_ref()
            .map(|node| RefMut::map((**node).borrow_mut(), |node| node.elem.as_mut().unwrap()))
    }

    pub fn peek_front_mut(&mut self) -> Option<RefMut<T>> {
        self.head
            .as_ref()
            .map(|node| RefMut::map((**node).borrow_mut(), |node| node.elem.as_mut().unwrap()))
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter(self)
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

pub struct IntoIter<T>(List<T>);

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.0.pop_front()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }
}

// utils - assuming link is not None

pub fn get_next<T>(link: &Link<T>) -> Link<T> {
    (**link.as_ref().unwrap()).borrow().next()
}

// TODO: maybe we need to make the following to macros to better enforce isolation
// e.g. (**link.as_ref().unwrap()).borrow().elem()

pub fn get_elem<T>(link: &Link<T>) -> Ref<T> {
    Ref::map((**link.as_ref().unwrap()).borrow(), |borrow| borrow.elem())
}

pub fn take_elem<T>(link: &Link<T>) -> T {
    let mut node = (**link.as_ref().unwrap()).borrow_mut();
    node.elem.take().unwrap()
}

// same for this function
pub fn get_elem_mut<T>(link: &Link<T>) -> RefMut<T> {
    RefMut::map((**link.as_ref().unwrap()).borrow_mut(), |borrow_mut| {
        borrow_mut.elem_mut()
    })
}

pub fn set_elem<T>(link: &Link<T>, new_val: T) {
    let mut node = (**link.as_ref().unwrap()).borrow_mut();
    node.elem.replace(new_val);
}

pub fn link_eq<T>(link_a: &Link<T>, link_b: &Link<T>) -> bool {
    link_a.is_some()
        && link_b.is_some()
        && Rc::ptr_eq(link_a.as_ref().unwrap(), link_b.as_ref().unwrap())
}

// tests

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop_front(), None);

        // Populate list
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(3));
        assert_eq!(list.pop_front(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_front(4);
        list.push_front(5);

        // Check normal removal
        assert_eq!(list.pop_front(), Some(5));
        assert_eq!(list.pop_front(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);

        // ---- back -----

        // Check empty list behaves right
        assert_eq!(list.pop_back(), None);

        // Populate list
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push_back(4);
        list.push_back(5);

        // Check normal removal
        assert_eq!(list.pop_back(), Some(5));
        assert_eq!(list.pop_back(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn peek() {
        let mut list = List::new();
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
        assert!(list.peek_front_mut().is_none());
        assert!(list.peek_back_mut().is_none());

        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        assert_eq!(&*list.peek_front().unwrap(), &3);
        assert_eq!(&mut *list.peek_front_mut().unwrap(), &mut 3);
        assert_eq!(&*list.peek_back().unwrap(), &1);
        assert_eq!(&mut *list.peek_back_mut().unwrap(), &mut 1);
    }

    #[test]
    fn into_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);

        let mut iter = list.into_iter();
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }
}

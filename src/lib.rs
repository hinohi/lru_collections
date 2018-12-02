#![feature(box_into_raw_non_null)]
use std::borrow::Borrow;
use std::boxed::Box;
use std::collections::HashMap;
use std::hash::Hash;
use std::ptr::NonNull;

struct MyLinkedList<T> {
    head: Option<NonNull<Node<T>>>,
    tail: Option<NonNull<Node<T>>>,
}

struct Node<T> {
    next: Option<NonNull<Node<T>>>,
    prev: Option<NonNull<Node<T>>>,
    element: T,
}

pub struct LruHashMap<K, V>
where
    K: Hash + Eq,
{
    map: HashMap<K, Node<V>>,
    list: MyLinkedList<V>,
}

impl<T> MyLinkedList<T> {
    fn new() -> MyLinkedList<T> {
        MyLinkedList {
            head: None,
            tail: None,
        }
    }

    fn push_front_node(&mut self, mut node: Box<Node<T>>) {
        node.next = self.head;
        node.prev = None;
        unsafe {
            let node = Some(Box::into_raw_non_null(node));

            match self.head {
                None => self.tail = node,
                Some(mut head) => head.as_mut().prev = node,
            }

            self.head = node;
        }
    }

    fn drop_back_node(&mut self) {
        if self.tail.is_none() {
            return;
        }

        unsafe {
            let node = self.tail.unwrap().as_ptr();
            self.tail = (*node).prev;

            match self.tail {
                None => self.head = None,
                Some(mut tail) => tail.as_mut().next = None,
            }
            // TODO node がリークしてない？
        }
    }

    unsafe fn unlink_and_push_front(&mut self, mut node: Box<Node<T>>) {
        let node = node.as_mut();

        match node.prev {
            Some(mut prev) => prev.as_mut().next = node.next.clone(),
            // this node is the head node
            // nothing to do
            None => return,
        }

        match node.next {
            Some(mut next) => next.as_mut().prev = node.prev.clone(),
            // this node is the tail node
            // node.prev is Some<_> in this branch
            None => self.tail = node.prev.clone(),
        };

        node.next = self.head;
        node.prev = None;

        let node = Some(node.into());
        self.head.unwrap().as_mut().prev = node;
        self.head = node;
    }
}

impl<K, V> LruHashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> LruHashMap<K, V> {
        LruHashMap {
            map: HashMap::new(),
            list: MyLinkedList::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

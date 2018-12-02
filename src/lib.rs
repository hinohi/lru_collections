#![feature(box_into_raw_non_null)]
use std::borrow::Borrow;
use std::boxed::Box;
use std::collections::HashMap;
use std::hash::Hash;
use std::ptr::NonNull;

struct MyLinkedList<K, V> {
    head: Option<NonNull<Node<K, V>>>,
    tail: Option<NonNull<Node<K, V>>>,
}

struct Node<K, V> {
    next: Option<NonNull<Node<K, V>>>,
    prev: Option<NonNull<Node<K, V>>>,
    key: NonNull<K>,
    value: V,
}

pub struct LruHashMap<K, V>
where
    K: Hash + Eq,
{
    max_size: usize,
    map: HashMap<K, NonNull<Node<K, V>>>,
    list: MyLinkedList<K, V>,
}

impl<K, T> Node<K, T> {
    fn new(key: *mut K, value: T) -> Node<K, T> {
        Node {
            next: None,
            prev: None,
            key: NonNull::new(key).unwrap(),
            value,
        }
    }
}

impl<K, V> MyLinkedList<K, V> {
    fn new() -> MyLinkedList<K, V> {
        MyLinkedList {
            head: None,
            tail: None,
        }
    }

    fn push_front_node(&mut self, mut node: Box<Node<K, V>>) {
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

    fn drop_back_node(&mut self) -> Option<Box<Node<K, V>>> {
        if self.tail.is_none() {
            return None;
        }

        unsafe {
            let node = self.tail.unwrap().as_ptr();
            self.tail = (*node).prev;

            match self.tail {
                None => self.head = None,
                Some(mut tail) => tail.as_mut().next = None,
            }

            Some(Box::from_raw(node))
        }
    }

    unsafe fn unlink_and_push_front(&mut self, mut node: Box<Node<K, V>>) {
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
    pub fn new(max_size: usize) -> LruHashMap<K, V> {
        LruHashMap {
            max_size,
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

    pub fn insert(&mut self, k: K, v: V) {
        // TODO use entry API
        if self.map.contains_key(&k) {
            unsafe {
                let mut node = self.map[&k];
                self.list
                    .unlink_and_push_front(Box::from_raw(node.as_ptr()));
                let node = node.as_mut();
                node.value = v;
            }
            return;
        }

        let mut k = k;
        let mut node = Node::new(&mut k as *mut K, v);
        self.map.insert(k, NonNull::new(&mut node).unwrap());
        self.list.push_front_node(Box::new(node));

        // TODO drop
    }

    pub fn get<Q>(&mut self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let ptr = match self.map.get(k) {
            None => return None,
            Some(ptr) => ptr,
        };
        unsafe {
            let node = Box::from_raw(ptr.as_ptr());
            self.list.unlink_and_push_front(node);
            Some(&ptr.as_ref().value)
        }
    }
}

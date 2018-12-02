#![feature(box_into_raw_non_null)]
use std::borrow::Borrow;
use std::boxed::Box;
use std::collections::HashMap;
use std::fmt::{self, Debug, Formatter};
use std::hash::Hash;
use std::mem;
use std::ptr::NonNull;

struct MyLinkedList<K, V> {
    head: Option<NonNull<Node<K, V>>>,
    tail: Option<NonNull<Node<K, V>>>,
}

struct Node<K, V> {
    next: Option<NonNull<Node<K, V>>>,
    prev: Option<NonNull<Node<K, V>>>,
    key: K,
    value: V,
}

pub struct LruHashMap<K, V>
where
    K: Hash + Eq + Clone,
{
    max_size: usize,
    map: HashMap<K, NonNull<Node<K, V>>>,
    list: MyLinkedList<K, V>,
}

impl<K, T> Node<K, T> {
    fn new(key: K, value: T) -> Node<K, T> {
        Node {
            next: None,
            prev: None,
            key,
            value,
        }
    }
}

impl<K, V> Debug for MyLinkedList<K, V>
where
    K: Debug,
{
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let mut i = 0;
        let mut now = self.head;
        writeln!(f, "MyLinkedList")?;
        while now.is_some() {
            let node = now.unwrap();
            unsafe {
                writeln!(f, "  {} = {:?}", i, node.as_ref().key)?;
                now = node.as_ref().next;
            }
            i += 1;
        }
        Ok(())
    }
}

impl<K, V> MyLinkedList<K, V> {
    fn new() -> MyLinkedList<K, V> {
        MyLinkedList {
            head: None,
            tail: None,
        }
    }

    fn push_front_node(&mut self, mut node: Box<Node<K, V>>) -> NonNull<Node<K, V>> {
        unsafe {
            node.next = self.head;
            node.prev = None;

            let node = Some(Box::into_raw_non_null(node));

            match self.head {
                None => self.tail = node,
                Some(mut head) => head.as_mut().prev = node,
            }

            self.head = node;
        }
        self.head.unwrap()
    }

    fn pop_back_node(&mut self) -> Option<Box<Node<K, V>>> {
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

    unsafe fn unlink_and_push_front(&mut self, node: *mut Node<K, V>) {
        let node = node.as_mut().unwrap();
        match node.prev {
            Some(mut prev) => {
                prev.as_mut().next = node.next.clone();
            }
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
    K: Hash + Eq + Clone,
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

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        // TODO use entry API
        if self.map.contains_key(&k) {
            unsafe {
                let mut node = self.map[&k];
                self.list.unlink_and_push_front(node.as_ptr());
                let node = node.as_mut();
                return Some(mem::replace(&mut node.value, v));
            }
        }

        // insert new node
        let node = Node::new(k.clone(), v);
        let ptr = self.list.push_front_node(Box::new(node));
        self.map.insert(k, ptr);

        // check size
        if self.max_size == 0 || self.map.len() <= self.max_size {
            return None;
        }

        // drop oldest node
        let tail = self.list.pop_back_node().unwrap();
        let key = tail.key;
        self.map.remove(&key);
        None
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
            self.list.unlink_and_push_front(ptr.as_ptr());
            Some(&ptr.as_ref().value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke() {
        let mut m = LruHashMap::new(10);
        assert_eq!(m.get("a"), None);
        assert_eq!(m.insert("a".to_string(), "A".to_string()), None);
        assert_eq!(m.get("a"), Some(&"A".to_string()));
        assert_eq!(
            m.insert("a".to_string(), "AA".to_string()),
            Some("A".to_string())
        );
        assert_eq!(m.get("a"), Some(&"AA".to_string()));
        assert_eq!(m.insert("b".to_string(), "B".to_string()), None);
        assert_eq!(m.get("a"), Some(&"AA".to_string()));
        assert_eq!(m.get("b"), Some(&"B".to_string()));
    }

    #[test]
    fn lru() {
        let mut m = LruHashMap::new(2);
        m.insert(1, 10);
        m.insert(2, 20);
        assert_eq!(m.len(), 2);
        m.insert(3, 30);
        assert_eq!(m.len(), 2);
        assert_eq!(m.get(&1), None);
        assert_eq!(m.get(&2), Some(&20));
        m.insert(4, 40);
        assert_eq!(m.len(), 2);
        assert_eq!(m.get(&3), None);
        assert_eq!(m.get(&2), Some(&20));
        assert_eq!(m.get(&4), Some(&40));
    }

    #[test]
    fn unlimited() {
        let mut m = LruHashMap::new(0);
        for i in 0..100000 {
            assert_eq!(m.insert(i, i), None);
        }
        for i in (0..100000).rev() {
            assert_eq!(m.get(&i), Some(&i));
        }
    }
}

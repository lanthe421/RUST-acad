use std::sync::{Arc, Mutex};

type Link<T> = Option<Arc<Mutex<Node<T>>>>;

struct Node<T> {
    val: T,
    prev: Link<T>,
    next: Link<T>,
}

struct Inner<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
}

/// Thread-safe doubly linked list.
/// Clone is cheap (Arc) — all clones share the same list.
/// No external Arc<Mutex<T>> needed.
#[derive(Clone)]
pub struct LinkedList<T>(Arc<Mutex<Inner<T>>>);

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(Inner { head: None, tail: None, len: 0 })))
    }

    pub fn len(&self) -> usize { self.0.lock().unwrap().len }
    pub fn is_empty(&self) -> bool { self.len() == 0 }

    pub fn push_back(&self, val: T) {
        let mut inner = self.0.lock().unwrap();
        let node = Arc::new(Mutex::new(Node { val, prev: inner.tail.clone(), next: None }));
        match inner.tail.replace(Arc::clone(&node)) {
            None => inner.head = Some(node),
            Some(old) => old.lock().unwrap().next = Some(node),
        }
        inner.len += 1;
    }

    pub fn push_front(&self, val: T) {
        let mut inner = self.0.lock().unwrap();
        let node = Arc::new(Mutex::new(Node { val, prev: None, next: inner.head.clone() }));
        match inner.head.replace(Arc::clone(&node)) {
            None => inner.tail = Some(node),
            Some(old) => old.lock().unwrap().prev = Some(node),
        }
        inner.len += 1;
    }

    pub fn pop_front(&self) -> Option<T> {
        let mut inner = self.0.lock().unwrap();
        inner.head.take().map(|node| {
            let next = node.lock().unwrap().next.take();
            match next.clone() {
                Some(ref h) => h.lock().unwrap().prev = None,
                None => inner.tail = None,
            }
            inner.head = next;
            inner.len -= 1;
            Arc::try_unwrap(node).ok().unwrap().into_inner().unwrap().val
        })
    }

    pub fn pop_back(&self) -> Option<T> {
        let mut inner = self.0.lock().unwrap();
        inner.tail.take().map(|node| {
            let prev = node.lock().unwrap().prev.take();
            match prev.clone() {
                Some(ref t) => t.lock().unwrap().next = None,
                None => inner.head = None,
            }
            inner.tail = prev;
            inner.len -= 1;
            Arc::try_unwrap(node).ok().unwrap().into_inner().unwrap().val
        })
    }

    pub fn to_vec(&self) -> Vec<T> where T: Clone {
        let inner = self.0.lock().unwrap();
        let mut result = Vec::with_capacity(inner.len);
        let mut cur = inner.head.clone();
        while let Some(node) = cur {
            let n = node.lock().unwrap();
            result.push(n.val.clone());
            cur = n.next.clone();
        }
        result
    }
}

fn main() {
    // Single-threaded
    let list = LinkedList::new();
    list.push_back(1);
    list.push_back(2);
    list.push_front(0);
    println!("{:?}", list.to_vec());        // [0, 1, 2]
    println!("{:?}", list.pop_front());     // Some(0)
    println!("{:?}", list.pop_back());      // Some(2)

    // Multi-threaded — just clone the list, no Arc<Mutex> needed on top
    let list: LinkedList<i32> = LinkedList::new();
    let handles: Vec<_> = (0..4).map(|i| {
        let list = list.clone(); // cheap Arc clone, same underlying list
        std::thread::spawn(move || list.push_back(i))
    }).collect();
    for h in handles { h.join().unwrap(); }
    println!("len = {}", list.len()); // 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop() {
        let l = LinkedList::new();
        l.push_back(1); l.push_back(2); l.push_front(0);
        assert_eq!(l.to_vec(), vec![0, 1, 2]);
        assert_eq!(l.pop_front(), Some(0));
        assert_eq!(l.pop_back(), Some(2));
        assert_eq!(l.pop_front(), Some(1));
        assert_eq!(l.pop_front(), None);
    }

    #[test]
    fn len_empty() {
        let l: LinkedList<i32> = LinkedList::new();
        assert!(l.is_empty());
        l.push_back(1);
        assert_eq!(l.len(), 1);
        l.pop_front();
        assert!(l.is_empty());
    }

    #[test]
    fn multithreaded() {
        let list: LinkedList<i32> = LinkedList::new();
        let handles: Vec<_> = (0..8).map(|i| {
            let list = list.clone();
            std::thread::spawn(move || list.push_back(i))
        }).collect();
        for h in handles { h.join().unwrap(); }
        assert_eq!(list.len(), 8);
    }
}

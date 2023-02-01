use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ptr::NonNull;

struct Node<K, V> {
    key: K,
    value: V,
    prev: Option<NonNull<Node<K, V>>>,
    next: Option<NonNull<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            prev: None,
            next: None,
        }
    }
}

pub struct Cache<K, V> {
    map: HashMap<K, NonNull<Node<K, V>>>,
    head: Option<NonNull<Node<K, V>>>,
    tail: Option<NonNull<Node<K, V>>>,
    capacity: usize,
}

impl<K, V> Cache<K, V>
where
    K: std::cmp::Eq + std::hash::Hash + Clone,
{
    pub fn new(capacity: usize) -> Self {
        Self {
            map: HashMap::new(),
            head: None,
            tail: None,
            capacity,
        }
    }

    fn remove_node(&mut self, node: &mut Node<K, V>) {
        let prev = node.prev.take();
        let next = node.next.take();

        match (prev, next) {
            (None, None) => {
                self.head = None;
                self.tail = None;
            }
            (None, Some(next)) => {
                self.head = Some(next);
                unsafe { (*next.as_ptr()).prev = None };
            }
            (Some(prev), None) => {
                self.tail = Some(prev);
                unsafe { (*prev.as_ptr()).next = None };
            }
            (Some(prev), Some(next)) => unsafe {
                (*prev.as_ptr()).next = Some(next);
                (*next.as_ptr()).prev = Some(prev);
            },
        }
    }

    fn add_node(&mut self, node: &mut Node<K, V>) {
        node.prev = None;
        node.next = self.head;

        if let Some(head) = self.head {
            unsafe { (*head.as_ptr()).prev = Some(node.into()) };
        }

        self.head = Some(node.into());

        if self.tail.is_none() {
            self.tail = Some(node.into());
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(node) = self.map.get(key) {
            let node = unsafe { node.as_ptr().as_mut().unwrap() };
            self.remove_node(node);
            self.add_node(node);
            Some(&node.value)
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.map.entry(key.clone()) {
            Entry::Occupied(entry) => {
                let node = unsafe { entry.get().as_ptr().as_mut().unwrap() };
                node.value = value;
                self.remove_node(node);
                self.add_node(node);
            }
            Entry::Vacant(_entry) => {
                if self.map.len() == self.capacity {
                    let tail = self.tail.take().unwrap();
                    let node = unsafe { tail.as_ptr().as_mut().unwrap() };
                    self.remove_node(node);
                    self.map.remove(&node.key);
                }

                let node = Box::new(Node::new(key.clone(), value));

                let node = unsafe {
                    let ptr = Box::into_raw(node);
                    &mut *ptr
                };

                let node = NonNull::new(node).unwrap();
                self.map.insert(key, node);
                self.add_node(unsafe { node.as_ptr().as_mut().unwrap() });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache() {
        let mut cache = Cache::new(3);

        cache.insert(1, "aws".to_owned());
        assert_eq!(cache.get(&1), Some(&"aws".to_owned()));

        cache.insert(2, "gcp".to_owned());
        cache.insert(3, "azure".to_owned());
        assert_eq!(cache.get(&3), Some(&"azure".to_owned()));

        cache.insert(4, "vmware".to_owned());
        assert_eq!(cache.get(&2), Some(&"gcp".to_owned()));
        assert_eq!(cache.get(&1), None);

        cache.insert(5, "val".to_owned());
        assert_eq!(cache.get(&5), Some(&"val".to_owned()));
        assert_eq!(cache.get(&4), Some(&"vmware".to_owned()));
        assert_eq!(cache.get(&3), None);
        assert_eq!(cache.get(&2), Some(&"gcp".to_owned()));
        assert_eq!(cache.get(&1), None);
    }
}

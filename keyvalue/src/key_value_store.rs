// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

/// Use the Observer trait to provide an implementation
/// to the [`InMemoryKeyValueStore](InMemoryKeyValueStore) if you need notifications
/// about field updates.
pub trait Observer<K, V> {
    fn on_set(&mut self, key: &K, value: &V);
}

/// Implementation of the in memory key value store
///
/// # Examples
///
/// ## Using the key value store with an observer
///
/// ```
/// use crate::keyvalue::key_value_store::InMemoryKeyValueStore;
/// use crate::keyvalue::key_value_store::Observer;
///
/// pub struct MyObserver {}
///
/// impl Observer<String, String> for MyObserver {
///    fn on_set(&mut self, key: &String, value: &String) {
///        assert_eq!(key, "key");
///        assert_eq!(value, "value");
///    }
///}
///
/// let my_observer = MyObserver {};
/// let mut store = InMemoryKeyValueStore::new(Some(my_observer));
/// store.set("key".into(), "value".into());
///
/// ```
///
/// ## Using the key value store without an observer
///
/// ```
/// use crate::keyvalue::key_value_store::InMemoryKeyValueStore;
/// use crate::keyvalue::key_value_store::Observer;
/// struct NotImplementedValueObserver;
///
/// impl<K, V> Observer<K, V> for NotImplementedValueObserver {
///     fn on_set(&mut self, _key: &K, _value: &V) {
///         todo!()
///     }
/// }
///
/// let mut store = InMemoryKeyValueStore::<String, String, NotImplementedValueObserver>::new(None);
/// store.set("key".into(), "value".into());
/// assert_eq!(store.get(&"key".to_string()), Some(&"value".to_string()));
///
/// ```
///
pub struct InMemoryKeyValueStore<K, V, O> {
    store: HashMap<K, V>,
    observer: Option<O>,
}

impl<K, V, O> InMemoryKeyValueStore<K, V, O>
where
    O: Observer<K, V>,
    K: Hash + Eq,
{
    /// Creates a new in-memory key-value store
    ///
    /// # Arguments
    /// * [`observer`](Observer) - The observer to be called on each field update, if any
    ///
    /// # Returns
    /// * [`InMemoryKeyValueStore`](InMemoryKeyValueStore)
    pub fn new(observer: Option<O>) -> InMemoryKeyValueStore<K, V, O> {
        InMemoryKeyValueStore { store: HashMap::new(), observer }
    }

    /// Retrieves a value from the store
    ///
    /// # Arguments
    /// * `key` - The key to retrieve the value for
    ///
    /// # Returns
    /// * [`Option<V>`](std::option) for the given key, if any
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.store.get(key)
    }

    /// Sets a value in the store
    ///
    /// # Arguments
    /// * `key` - The key to set the value for
    /// * `value` - The value to set
    ///
    /// > **Note** calls the observer, if any
    pub fn set(&mut self, key: K, value: V) {
        if let Some(ref mut observer) = self.observer {
            observer.on_set(&key, &value);
            self.store.insert(key, value);
        } else {
            self.store.insert(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key_value_store::InMemoryKeyValueStore;
    use std::{
        collections::HashMap,
        sync::{atomic::AtomicUsize, Arc},
    };

    struct NotImplementedValueObserver;

    impl<K, V> Observer<K, V> for NotImplementedValueObserver {
        fn on_set(&mut self, _key: &K, _value: &V) {
            todo!()
        }
    }

    fn setup_none_observer<K, V>() -> InMemoryKeyValueStore<K, V, NotImplementedValueObserver>
    where
        K: Eq + std::hash::Hash,
    {
        InMemoryKeyValueStore::<K, V, NotImplementedValueObserver>::new(None)
    }

    #[test]
    fn test_key_value_store() {
        let mut store = setup_none_observer::<String, String>();
        store.set("key".into(), "value".into());
        assert_eq!(store.get(&"key".to_string()), Some(&"value".to_string()));
    }

    #[test]
    fn test_key_value_store_with_custom_struct() {
        let mut map = HashMap::new();
        map.insert("test".to_string(), "value".to_string());

        #[derive(Clone, Debug, Eq, PartialEq)]
        pub struct Test {
            name: String,
            age: i32,
            metadata: HashMap<String, String>,
            data: Vec<u8>,
            links: Vec<Test>,
        }

        let inner_test = Test {
            name: "test".to_string(),
            age: 42,
            metadata: map.clone(),
            data: vec![1, 2, 3],
            links: vec![Test {
                name: "test".to_string(),
                age: 42,
                metadata: map,
                data: vec![1, 2, 3],
                links: vec![],
            }],
        };

        let mut store = setup_none_observer::<String, Test>();
        store.set("key".to_string(), inner_test.clone());
        let result = store.get(&"key".to_string()).unwrap();
        assert_eq!(result.name, inner_test.name);
        assert_eq!(result.age, inner_test.age);
        assert_eq!(result.metadata, inner_test.metadata);
        assert_eq!(result.data, inner_test.data);
        assert_eq!(result.links.len(), 1);
    }

    #[test]
    fn test_key_value_store_with_observer() {
        #[derive(Clone, Copy)]
        pub struct MyObserver {}

        impl Observer<String, String> for MyObserver {
            fn on_set(&mut self, key: &String, value: &String) {
                assert_eq!(key, "key");
                assert_eq!(value, "value");
            }
        }

        let my_observer = MyObserver {};
        let mut store = InMemoryKeyValueStore::new(Some(my_observer));
        store.set("key".into(), "value".into());
    }

    #[test]
    fn test_observer_gets_notified_twice_when_setting_an_unchanged_value() {
        #[derive(Clone)]
        pub struct MyObserver {
            counter: Arc<AtomicUsize>,
        }

        impl Observer<String, String> for MyObserver {
            fn on_set(&mut self, key: &String, value: &String) {
                assert_eq!(key, "key");
                assert_eq!(value, "value");
                self.counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        let my_observer = MyObserver { counter: Arc::new(AtomicUsize::new(0)) };
        let mut store = InMemoryKeyValueStore::new(Some(my_observer.clone()));
        store.set("key".into(), "value".into());
        store.set("key".into(), "value".into());

        assert_eq!(my_observer.counter.load(std::sync::atomic::Ordering::Relaxed), 2);
    }
}

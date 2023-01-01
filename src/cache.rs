use std::{
    collections::{BinaryHeap, HashMap},
    ops::Add,
    time::{Duration, SystemTime},
};

use crate::providers::VersionInfo;

pub trait Clock<T> {
    fn now(&self) -> T;
}

pub struct SystemClock;

impl Clock<SystemTime> for SystemClock {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }
}

pub struct ReleaseCache<T: Add<Duration> + Ord, C: Clock<T>> {
    cache: HashMap<String, Vec<VersionInfo>>,
    expiry: BinaryHeap<(T, String)>,
    clock: C,
}

impl<T: Add<Duration, Output = T> + Ord, C: Clock<T>> ReleaseCache<T, C> {
    pub fn new(clock: C) -> Self {
        Self {
            cache: HashMap::new(),
            expiry: BinaryHeap::new(),
            clock,
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&Vec<VersionInfo>> {
        self.cache.get(key)
    }

    pub fn insert(&mut self, key: String, releases: Vec<VersionInfo>, cache_duration: Duration) {
        self.cache.insert(key.clone(), releases);
        self.expiry.push((self.clock.now() + cache_duration, key));
    }

    pub fn expire(&mut self) {
        while let Some(item) = self.expiry.peek() {
            if item.0 > self.clock.now() {
                break;
            }
            self.cache.remove(&item.1);
            self.expiry.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, ops::Add, rc::Rc, time::Duration};

    use super::{Clock, ReleaseCache};

    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    struct Timestamp(u64);

    impl Add<Duration> for Timestamp {
        type Output = Timestamp;

        fn add(self, rhs: Duration) -> Self::Output {
            Timestamp(self.0 + rhs.as_secs())
        }
    }

    #[derive(Clone)]
    struct ControlledClock {
        timestamp: Rc<RefCell<u64>>,
    }

    impl Clock<Timestamp> for ControlledClock {
        fn now(&self) -> Timestamp {
            Timestamp(*self.timestamp.borrow())
        }
    }

    impl ControlledClock {
        fn new() -> Self {
            Self {
                timestamp: Rc::new(RefCell::new(0)),
            }
        }

        fn advance(&mut self, secs: u64) {
            *self.timestamp.borrow_mut() += secs;
        }
    }

    #[test]
    fn test_release_cache_caching_behaviour() {
        let mut clock = ControlledClock::new();
        let mut cache = ReleaseCache::new(clock.clone());

        assert!(!cache.contains_key("key"));
        assert!(cache.get("key").is_none());

        cache.insert("key".into(), vec![], Duration::from_secs(10));
        assert!(cache.contains_key("key"));
        assert_eq!(cache.get("key"), Some(&vec![]));

        cache.expire();
        assert!(cache.contains_key("key"));
        assert_eq!(cache.get("key"), Some(&vec![]));

        clock.advance(20);
        cache.expire();
        assert!(!cache.contains_key("key"));
        assert!(cache.get("key").is_none());
    }
}

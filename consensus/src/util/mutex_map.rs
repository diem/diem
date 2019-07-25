use futures::compat::Future01CompatExt;
use std::{
    collections::HashMap,
    hash::Hash,
    sync::{self, Arc},
};

type AsyncMutex = futures_locks::Mutex<()>;

impl<T> Drop for MutexMapGuard<T>
where
    T: Hash + Eq + Clone,
{
    fn drop(&mut self) {
        // Drop guard: release lock, decrement internal reference counter on lock
        self.mutex_guard
            .take()
            .expect("ExistGuard should have mutex_guard on drop");
        self.owner.guard_dropped(&self.key);
    }
}

/// MutexMap acts as a map of mutexes, allowing lock based on some key
///
/// More specifically MutexMap::lock(key) returns lock guard, that is guaranteed to be exclusive
/// for given key. Until returned guard is dropped, consequent calls to MutexMap::lock with same
/// key going to be blocked.
///
/// MutexMap is from a 'future locks' family, meaning that lock() method does not block thread,
/// but instead returns Future(implicitly, through 'async') that is only fulfilled when lock
/// can be acquired.
pub struct MutexMap<T>(Arc<sync::Mutex<HashMap<T, AsyncMutex>>>)
where
    T: Hash + Eq + Clone;

/// This guard is returned by MutexMap::lock, dropping this guard releases lock
pub struct MutexMapGuard<T>
where
    T: Hash + Eq + Clone,
{
    mutex_guard: Option<futures_locks::MutexGuard<()>>,
    owner: MutexMap<T>,
    key: T,
}

// Implementation overview:
// MutexMap maintains map of future mutexes
// Map itself is protected under regular, non-future mutex, to reduce code complexity.
// Operation on this map is fast, so having traditional lock does not have impact.
//
// Only private methods get_or_create_mutex and guard_dropped acquire lock on internal map
impl<T> MutexMap<T>
where
    T: Hash + Eq + Clone,
{
    /// Creates new, empty MutexMap
    pub fn new() -> MutexMap<T> {
        MutexMap(Arc::new(sync::Mutex::new(HashMap::new())))
    }

    /// async method is fulfilled when lock can be acquired
    /// Lock is released when returned guard is dropped
    pub async fn lock(&self, key: T) -> MutexMapGuard<T> {
        let mutex_fut = self.get_or_create_mutex(key.clone());
        let mutex_guard = mutex_fut.compat().await.unwrap();
        let owner = Self(Arc::clone(&self.0));
        MutexMapGuard {
            mutex_guard: Some(mutex_guard),
            owner,
            key,
        }
    }

    /// Creates new empty lock_set for given map.
    /// Use this if you want to acquire multiple locks in a loop in single task
    pub fn new_lock_set(&self) -> LockSet<T> {
        LockSet::new(self)
    }

    fn get_or_create_mutex(&self, key: T) -> futures_locks::MutexFut<()> {
        let mut map = self.0.lock().unwrap();
        let e = map.entry(key);
        let mutex_ref = e.or_insert_with(|| AsyncMutex::new(()));
        // This one is async lock, it will return future that keeps reference on underlining mutex
        // We can't return mutex_ref itself, because
        mutex_ref.lock()
    }

    fn guard_dropped(&self, key: &T) {
        let mut map = self.0.lock().unwrap();
        // This is a bit obscure, but we have to remove mutex from map before we can try
        // to unwrap it. If unwrap fails(pending references exist), we put mutex back into map
        // Since we are holding lock on map, this remove-insert does not introduces races
        // This is ok from performance point of view too:
        // removing lock is a 'happy path': if there is no pending lock,
        // then we will be able to unwrap, and in most cases won't need to re-insert mutex
        let mutex = map.remove(&key);
        // It is possible that mutex is not in map, if multiple guards dropped concurrently
        if let Some(mutex) = mutex {
            if let Err(cloned_mutex) = mutex.try_unwrap() {
                map.insert(key.clone(), cloned_mutex);
            }
        }
    }

    #[cfg(test)]
    pub fn held_locks_count(&self) -> usize {
        self.0.lock().unwrap().len()
    }
}

/// LockSet represents set of acquired locks
/// In addition to keeping multiple locks, LockSet protects from deadlocks
/// LockSet::lock will fail if lock is already acquired for this lock set
///
/// LockSet is useful if you acquire multiple locks in a loop, to make sure there is no deadlock.
/// It does not help if independent parts of code acquire locks and don't use LockSet.
///
/// Ideally we would want to track locks per task
/// Unfortunately, currently tokio does not support TaskLocal's, so we have to have
/// separated object to collect locks
pub struct LockSet<'a, T>(&'a MutexMap<T>, HashMap<T, MutexMapGuard<T>>)
where
    T: Clone + Eq + Hash;

impl<'a, T> LockSet<'a, T>
where
    T: Clone + Eq + Hash,
{
    fn new(mutex_map: &'a MutexMap<T>) -> LockSet<'a, T> {
        LockSet(mutex_map, HashMap::new())
    }

    /// Acquires lock for given object, fails if lock was already acquired through same lock set
    pub async fn lock(&mut self, t: T) -> Result<(), ()> {
        if self.1.contains_key(&t) {
            return Err(());
        }
        let guard = self.0.lock(t.clone()).await;
        self.1.insert(t, guard);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::util::mutex_map::*;
    use futures::{
        channel::mpsc, executor::block_on, stream::StreamExt, FutureExt, SinkExt, TryFutureExt,
    };
    use std::{mem, sync::Arc, thread, time::Duration};
    use tokio::runtime;

    #[test()]
    pub fn test_mutex_map() {
        let map = Arc::new(MutexMap::new());
        let (sender, recv) = mpsc::unbounded();
        let guard_1 = block_on(lock_1(map.clone()));
        let mut rt1 = runtime::Builder::new().core_threads(1).build().unwrap();
        let mut rt2 = runtime::Builder::new().core_threads(1).build().unwrap();
        rt1.spawn(
            task2(map.clone(), sender.clone())
                .boxed()
                .unit_error()
                .compat(),
        );
        rt2.spawn(
            task1(map.clone(), sender.clone(), guard_1)
                .boxed()
                .unit_error()
                .compat(),
        );
        // Need to drop our reference to sender
        mem::drop(sender);
        let events: Vec<&'static str> = block_on(recv.collect());
        assert_eq!(
            vec!["Task 1 Locked 2", "Task 2 Locked 1", "Task 2 Locked 2",],
            events
        );
        // wait for runtime to shutdown so guard Drop releases memory in map
        block_on(rt1.shutdown_on_idle().compat()).unwrap();
        block_on(rt2.shutdown_on_idle().compat()).unwrap();
        // Verify that map does not hold locks in memory when they are released
        assert_eq!(0, map.held_locks_count());
    }

    async fn lock_1(map: Arc<MutexMap<usize>>) -> MutexMapGuard<usize> {
        map.lock(1).await
    }

    async fn task1(
        map: Arc<MutexMap<usize>>,
        mut actions: mpsc::UnboundedSender<&'static str>,
        _guard_1: MutexMapGuard<usize>,
    ) {
        // Sleep gives chance for task2 to acquire lock and fail test if lock_map does not work
        // correctly
        thread::sleep(Duration::from_millis(1));
        map.lock(2).await;
        actions.send("Task 1 Locked 2").await.unwrap();
        // Drops _guard_1 and releases 1 so that task2 can re-acquire this lock
    }

    async fn task2(map: Arc<MutexMap<usize>>, mut actions: mpsc::UnboundedSender<&'static str>) {
        map.lock(1).await;
        actions.send("Task 2 Locked 1").await.unwrap();
        map.lock(2).await;
        actions.send("Task 2 Locked 2").await.unwrap();
    }
}

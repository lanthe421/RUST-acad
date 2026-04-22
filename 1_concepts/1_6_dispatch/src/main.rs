fn main() {
    use std::borrow::Cow;

    trait Storage<K, V> {
        fn set(&mut self, key: K, val: V);
        fn get(&self, key: &K) -> Option<&V>;
        fn remove(&mut self, key: &K) -> Option<V>;
    }

    pub struct User {
        id: u64,
        email: Cow<'static, str>,
        activated: bool,
    }

    // --- Dynamic dispatch ---
    pub struct DynUserRepository {
        storage: Box<dyn Storage<u64, User>>,
    }

    impl DynUserRepository {
        fn add(&mut self, user: User) { self.storage.set(user.id, user); }
        fn get(&self, id: &u64) -> Option<&User> { self.storage.get(id) }
        fn update(&mut self, user: User) { self.storage.set(user.id, user); }
        fn remove(&mut self, id: &u64) -> Option<User> { self.storage.remove(id) }
    }

    // --- Static dispatch ---
    struct StaticUserRepository<S: Storage<u64, User>> {
        storage: S,
    }

    impl<S: Storage<u64, User>> StaticUserRepository<S> {
        fn add(&mut self, user: User) { self.storage.set(user.id, user); }
        fn get(&self, id: &u64) -> Option<&User> { self.storage.get(id) }
        fn update(&mut self, user: User) { self.storage.set(user.id, user); }
        fn remove(&mut self, id: &u64) -> Option<User> { self.storage.remove(id) }
    }

}
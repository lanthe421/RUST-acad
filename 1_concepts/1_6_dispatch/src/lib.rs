use std::borrow::Cow;

pub trait Storage<K, V> {
    fn set(&mut self, key: K, val: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K) -> Option<V>;
}

pub struct User {
    pub id: u64,
    pub email: Cow<'static, str>,
    pub activated: bool,
}

pub trait UserRepository {
    fn get(&self, id: u64) -> Option<&User>;
}

pub struct DynUserRepository {
    pub storage: Box<dyn Storage<u64, User>>,
}

impl DynUserRepository {
    pub fn add(&mut self, user: User) { self.storage.set(user.id, user); }
    pub fn get(&self, id: &u64) -> Option<&User> { self.storage.get(id) }
    pub fn update(&mut self, user: User) { self.storage.set(user.id, user); }
    pub fn remove(&mut self, id: &u64) -> Option<User> { self.storage.remove(id) }
}

pub struct StaticUserRepository<S: Storage<u64, User>> {
    pub storage: S,
}

impl<S: Storage<u64, User>> StaticUserRepository<S> {
    pub fn add(&mut self, user: User) { self.storage.set(user.id, user); }
    pub fn get(&self, id: &u64) -> Option<&User> { self.storage.get(id) }
    pub fn update(&mut self, user: User) { self.storage.set(user.id, user); }
    pub fn remove(&mut self, id: &u64) -> Option<User> { self.storage.remove(id) }
}

use derive_more::{From, Into};
use im::{HashMap, HashSet};
fn main() {}

#[derive(Debug, PartialEq, Eq, Clone, From, Into)]
struct UserId(u64);
#[derive(Debug, PartialEq, Eq, Clone, From, Into)]
struct UserNickname(String);

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: UserId,
    nickname: UserNickname,
}

trait UsersRepository {
    fn get_by_id(&self, id: u64) -> Option<&User>;
    fn get_by_ids(&self, ids: &[u64]) -> HashMap<u64, &User>;
    fn search_by_nickname(&self, query: impl AsRef<str>) -> HashSet<u64>;
}

struct InMemoryUsersRepository {
    users: HashMap<u64, User>,
}

impl InMemoryUsersRepository {
    fn new(users: impl IntoIterator<Item = User>) -> Self {
        Self {
            users: users.into_iter().map(|user| (user.id.0, user)).collect(),
        }
    }
}

impl UsersRepository for InMemoryUsersRepository {
    fn get_by_id(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    fn get_by_ids(&self, ids: &[u64]) -> HashMap<u64, &User> {
        ids.iter()
            .filter_map(|id| self.users.get(id).map(|user| (*id, user)))
            .collect()
    }

    fn search_by_nickname(&self, query: impl AsRef<str>) -> HashSet<u64> {
        self.users
            .values()
            .filter(|user| user.nickname.0.contains(query.as_ref()))
            .map(|user| user.id.0)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;

    fn repo() -> InMemoryUsersRepository {
        InMemoryUsersRepository::new([
            User {
                id: UserId(1),
                nickname: UserNickname("alice".to_string()),
            },
            User {
                id: UserId(2),
                nickname: UserNickname("bob".to_string()),
            },
            User {
                id: UserId(3),
                nickname: UserNickname("alice_smith".to_string()),
            },
        ])
    }

    #[test]
    fn get_by_id_found() {
        let r = repo();
        assert_eq!(r.get_by_id(1).map(|user| user.id.clone()), Some(UserId(1)));
    }

    #[test]
    fn get_by_id_not_found() {
        assert!(repo().get_by_id(99).is_none());
    }

    #[test]
    fn get_by_ids_returns_existing() {
        let r = repo();
        let result = r.get_by_ids(&[1, 3, 99]);
        assert_eq!(result.len(), 2);
        assert!(result.contains_key(&1));
        assert!(result.contains_key(&3));
        assert!(!result.contains_key(&99));
    }

    #[test]
    fn search_by_nickname_finds_matches() {
        let r = repo();
        let ids = r.search_by_nickname("alice");
        assert_eq!(ids, HashSet::from_iter([1u64, 3u64]));
    }

    #[test]
    fn search_by_nickname_no_matches() {
        assert!(repo().search_by_nickname(Cow::from("xyz")).is_empty());
    }
}

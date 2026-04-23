use im::HashMap;

fn main() {}

#[derive(Debug, Clone, PartialEq)]
struct User {
    id: u64,
    nickname: String,
}

trait UsersRepository {
    fn get_by_id(&self, id: u64) -> Option<&User>;
    fn get_by_ids(&self, ids: &[u64]) -> Vec<&User>;
    fn search_by_nickname(&self, query: &str) -> Vec<u64>;
}

struct InMemoryUsersRepository {
    users: HashMap<u64, User>,
}

impl InMemoryUsersRepository {
    fn new(users: impl IntoIterator<Item = User>) -> Self {
        Self {
            users: users.into_iter().map(|user| (user.id, user)).collect(),
        }
    }
}

impl UsersRepository for InMemoryUsersRepository {
    fn get_by_id(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    fn get_by_ids(&self, ids: &[u64]) -> Vec<&User> {
        ids.iter().filter_map(|id| self.users.get(id)).collect()
    }

    fn search_by_nickname(&self, query: &str) -> Vec<u64> {
        self.users
            .values()
            .filter(|user| user.nickname.contains(query))
            .map(|user| user.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo() -> InMemoryUsersRepository {
        InMemoryUsersRepository::new([
            User { id: 1, nickname: "alice".into() },
            User { id: 2, nickname: "bob".into() },
            User { id: 3, nickname: "alice_smith".into() },
        ])
    }

    #[test]
    fn get_by_id_found() {
        let r = repo();
        assert_eq!(r.get_by_id(1).map(|user| user.id), Some(1));
    }

    #[test]
    fn get_by_id_not_found() {
        assert!(repo().get_by_id(99).is_none());
    }

    #[test]
    fn get_by_ids_returns_existing() {
        let r = repo();
        let mut ids: Vec<u64> = r.get_by_ids(&[1, 3, 99]).iter().map(|user| user.id).collect();
        ids.sort();
        assert_eq!(ids, vec![1, 3]);
    }

    #[test]
    fn search_by_nickname_finds_matches() {
        let r = repo();
        let mut ids = r.search_by_nickname("alice");
        ids.sort();
        assert_eq!(ids, vec![1, 3]);
    }

    #[test]
    fn search_by_nickname_no_matches() {
        assert!(repo().search_by_nickname("xyz").is_empty());
    }
}

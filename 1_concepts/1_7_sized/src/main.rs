use std::borrow::Cow;
use std::collections::HashMap;

use step_1_6::{User, UserRepository};

struct CreateUser {
    id: u64,
    email: Cow<'static, str>,
}

trait Command {}
impl Command for CreateUser {}

#[derive(Debug)]
enum UserError {
    AlreadyExists,
}

trait CommandHandler<C: Command> {
    type Context: ?Sized;
    type Result;
    fn handle_command(&self, cmd: &C, ctx: &Self::Context) -> Self::Result;
}

impl CommandHandler<CreateUser> for User {
    type Context = dyn UserRepository;
    type Result = Result<(), UserError>;

    fn handle_command(&self, cmd: &CreateUser, repo: &Self::Context) -> Self::Result {
        if repo.get(cmd.id).is_some() {
            return Err(UserError::AlreadyExists);
        }
        Ok(())
    }
}

fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockRepo(HashMap<u64, User>);

    impl UserRepository for MockRepo {
        fn get(&self, id: u64) -> Option<&User> {
            self.0.get(&id)
        }
    }

    impl MockRepo {
        fn add(&mut self, user: User) -> Result<(), UserError> {
            if self.0.contains_key(&user.id) {
                return Err(UserError::AlreadyExists);
            }
            self.0.insert(user.id, user);
            Ok(())
        }
    }

    #[test]
    fn creates_new_user() {
        let repo = MockRepo(HashMap::new());
        let handler = User { id: 0, email: Cow::Borrowed(""), activated: false };
        let cmd = CreateUser { id: 1, email: Cow::Borrowed("a@b.com") };
        assert!(handler.handle_command(&cmd, &repo as &dyn UserRepository).is_ok());
    }

    #[test]
    fn fails_if_user_exists() {
        let mut repo = MockRepo(HashMap::new());
        repo.0.insert(1, User { id: 1, email: Cow::Borrowed("a@b.com"), activated: false });
        let handler = User { id: 0, email: Cow::Borrowed(""), activated: false };
        let cmd = CreateUser { id: 1, email: Cow::Borrowed("a@b.com") };
        assert!(matches!(
            handler.handle_command(&cmd, &repo as &dyn UserRepository),
            Err(UserError::AlreadyExists)
        ));
    }
}

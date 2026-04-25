use crate::post::Id;

mod post {
    #[derive(Clone, Debug, PartialEq)]
    pub struct Id(pub u64);

    #[derive(Clone, Debug, PartialEq)]
    pub struct Title(pub String);

    #[derive(Clone, Debug, PartialEq)]
    pub struct Body(pub String);
}
mod user {
    use derive_more::{From, Into};
    #[derive(Clone, Debug, PartialEq, From, Into)]
    pub struct Id(pub u64);
}

#[derive(Debug)]
pub struct New;
#[derive(Debug)]
pub struct Unmoderated;
#[derive(Debug)]
pub struct Published;
#[derive(Debug)]
pub struct Deleted;

#[derive(Clone, Debug)]
struct PostData {
    id: post::Id,
    user_id: user::Id,
    title: post::Title,
    body: post::Body,
}

#[derive(Debug)]
struct Post<State> {
    data: PostData,
    state: std::marker::PhantomData<State>,
}

impl Post<New> {
    fn new(id: post::Id, user_id: user::Id, title: post::Title, body: post::Body) -> Post<New> {
        Post {
            data: PostData {id, user_id, title, body},
            state: std::marker::PhantomData,
        }
    }
    fn publish(&self) -> Post<Unmoderated> {
        Post {
            data: self.data.clone(),
            state: std::marker::PhantomData,
        }
    }
}

impl Post<Unmoderated> {
    fn allow(&self) -> Post<Published> {
        Post {
            data: self.data.clone(),
            state: std::marker::PhantomData,
        }
    }
    fn deny(&self) -> Post<Deleted> {
        Post {
            data: self.data.clone(),
            state: std::marker::PhantomData,
        }
    }
}

impl Post<Published> {
    fn delete(&self) -> Post<Deleted> {
        Post {
            data: self.data.clone(),
            state: std::marker::PhantomData,
        }
    }
}

impl Debug for Post<Deleted> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Post deleted")
    }
}

fn main() {
    let post = Post::<New>::new(
        post::Id(564),
        user::Id(512),
        post::Title(String::from("Hello")),
        post::Body("Day".to_string()),
    );
    let deleted = post.publish().allow().delete();
    println!("{:?}",deleted);
    let post2 = Post::<New>::new(
        post::Id(2),
        user::Id(42),
        post::Title(String::from("Bye")),
        post::Body(String::from("World")),
    );
    let post2 = post2.publish().deny();
    println!("{:?}", post2);
    // let post2 = post2.publish().allow(); // Error
    // let post2 = post2.deny(); // Error, not compile
}
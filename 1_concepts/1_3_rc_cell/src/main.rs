use std::sync::{Arc, Mutex};

#[derive(Clone, Debug)]
struct GlobalStack<T> {
    data: Arc<Mutex<Vec<T>>>,
}

impl<T> GlobalStack<T> {
    fn new() -> Self {
        GlobalStack { data: Arc::new(Mutex::new(Vec::new())) }
    }

    fn push(&self, value: T) {
        self.data.lock().unwrap().push(value)
    }

    fn pop(&self) -> Option<T> {
        self.data.lock().unwrap().pop()
    }
}

fn main() {
    let stack = GlobalStack::new();
    let stack2 = stack.clone(); // same data, just another pointer

    stack.push(1);
    stack2.push(2);

    println!("{:?}", stack.pop());
    println!("{:?}", stack.pop());  
    println!("{:?}", stack.pop());  
    println!("{:?}", stack.pop());  
    println!("{:?}", stack)
}

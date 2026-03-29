Step 1: Concepts
================

__Estimated time__: 2 days

These steps describe common and necessary-to-know concepts for everyday programming in [Rust].

> ❗️Before completing this step you should complete all its sub-steps.

After doing them you should be able to answer the following questions:
- How do I recognize that data is allocated at the heap rather than at the stack? When data should be allocated at the heap?

Data is allocated at the heap when it is moved out of the current scope. Data should be allocated at the heap when it is too big to be allocated at the stack or when it needs to be shared between different parts of the program.

- What is copying and cloning data in [Rust]? What's the difference? When and why should I use them?

Copying data in Rust means that the data is copied from one location to another. Cloning data means that the data is copied and a new instance is created. The difference is that copying is a shallow copy, while cloning is a deep copy. Copying is used when the data is small and cheap to copy, while cloning is used when the data is large or expensive to copy. Copying is done with the `Copy` trait, while cloning is done with the `Clone` trait.

- How can a single piece of data be owned by multiple parts of program? When and why is this commonly required?

A single piece of data can be owned by multiple parts of a program through the use of references. This is commonly required when the data needs to be accessed by multiple parts of the program at the same time.

- How borrowing rules may be violated? In what price? When and why is this commonly required?

Borrowing rules may be violated when a reference is used after the data it references has been moved or dropped. The price paid is a compile-time error. This is commonly required when the program needs to ensure that data is not accessed after it has been freed.

- How to deal with owned and borrowed data simultaneously? When and why is this commonly required?

The main method is to use lifetime annotations (<'a>) to associate borrowed data with their proofs.

When required:
For optimization, own data but return references to it (without copying)

For temporary processing, own some data and borrow others

For parsing without copying, own a buffer and borrow references within it.

For shared access, one owner, many readers

Why it's needed: Rust requires explicit instructions about who is responsible for deleting data. In programs themselves, data often needs to be stored in one place, but read in many places—without borrowing, everything needs to be copied.


- How to share values between threads? What is `Send` and `Sync` markers? Why are they required, when should be used?

Basic tools to share:
Arc — for shared ownership (atomic reference counting)

Mutex / RwLock — for safe mutability

Channels (mpsc) — for transferring ownership

Typical pattern: Arc<Mutex<T>>

What are Send and Sync?
Send — a type can be moved to another thread

Sync — a type can be referenced by multiple threads simultaneously (&T is thread-safe)

Why are they required?
For memory safety—the compiler checks that data won't be corrupted by concurrent access (data races, double frees).

When to use
Almost never manually—the compiler implements them automatically for your types if all fields support them.
Manual implementation is only necessary in unsafe code.
Check—when writing generic multithreaded code, require Send + Sync for types.

- How do static and dynamic dispatches differ? Why do they exist? When and why should I choose between them?
Dynamic dispatch allows to delay the decision of which implementation to call until runtime. It's implemented using trait objects and virtual function tables. Static dispatch allows to make the decision at compile time. It's implemented using generics and monomorphization. Dynamic dispatch is slower because of the indirection, but more flexible. Static dispatch is faster but less flexible. Use dynamic dispatch when you need to store different types in the same collection or when you don't know the concrete type at compile time. Use static dispatch when you know the concrete type at compile time and want the best performance.

- Why `?Sized` types exist? How are they used? Why should I care about them?

`?Sized` is a special trait bound that allows a type to be unsized. It is used to allow generic types to be used with unsized types like `str` or `[T]`. You should care about them because they allow you to write more flexible generic code.

- Why phantom types exist? What problems do they solve?
Phantom types are used to encode additional information in the type system. They solve the problem of ensuring that certain invariants are maintained at compile time. For example, you can use a phantom type to ensure that a type is only used with a specific kind of data.
```rust
use std::marker::PhantomData;

struct Phantom<T> {
    _marker: PhantomData<T>,
}

impl<T> Phantom<T> {
    fn new() -> Self {
        Phantom {
            _marker: PhantomData,
        }
    }
}

The following articles may help you to sum up your experience:
- [Wrapper Types in Rust: Choosing Your Guarantees][1]
- [Rust, Builder Pattern, Trait Objects, `Box<T>` and `Rc<T>`][2]
- [Rust's Built-in Traits, the When, How & Why][3]




## Task

Provide your own implementation of [doubly linked list][11] data structure. It should be [thread safe][12] without a necessity to use explicit synchronization primitives (like `Arc<Mutex<T>>`) on top of it.

Prove your implementation correctness with tests. Provide both single-threaded and multi-threaded examples of usage.  




[Rust]: https://www.rust-lang.org

[1]: https://manishearth.github.io/blog/2015/05/27/wrapper-types-in-rust-choosing-your-guarantees
[2]: https://abronan.com/rust-trait-objects-box-and-rc
[3]: https://llogiq.github.io/2015/07/30/traits.html
[11]: https://en.wikipedia.org/wiki/Doubly_linked_list
[12]: https://en.wikipedia.org/wiki/Thread_safety

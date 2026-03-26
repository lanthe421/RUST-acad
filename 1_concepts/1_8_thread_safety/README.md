Step 1.8: Thread safety
=======================

__Estimated time__: 1 day

[Rust] has [`Send`] and [`Sync`] marker traits which are fundamental for concurrency and thread safety story in [Rust] and represent one of [fearless concurrency][2] corner stones (which allow to [avoid data races][1] at compile time).

To better understand [`Send`]/[`Sync`]'s purpose, design, limitations and use cases, read through:
- [Official `Send` docs][`Send`]
- [Official `Sync` docs][`Sync`]
- [Rust Book: 16.4. Extensible Concurrency with the Sync and Send Traits][3]
- [Rustonomicon: 8.2. Send and Sync][4]
- [Huon Wilson: Some notes on Send and Sync][5]
- [Piotr Sarnacki: Arc and Mutex in Rust][9]
- [nyanpasu64: An unsafe tour of Rust's Send and Sync][6]
- [Josh Haberman: Thread Safety in C++ and Rust][7]
- [Cliff L. Biffle: Safely writing code that isn't thread-safe][8]
- [Louis Dureuil: Too dangerous for C++][10]
- [Cuong Le: This Send/Sync Secret Separates Professional From Amateur Rust Developers][11]




## Task

Implement the following types, which meet conditions:
1. `OnlySync` is `Sync`, but `!Send`.
2. `OnlySend` is `Send`, but `!Sync`.
3. `SyncAndSend` is both `Sync` and `Send`.
4. `NotSyncNotSend` is both `!Sync` and `!Send`.

All inner details of implementation are on your choice.

Play with these types from multiple threads to see how compile time [fearless concurrency][2] works in practice.




## Questions

After completing everything above, you should be able to answer (and understand why) the following questions:
- What does "fearless concurrency" mean in [Rust]? With which mechanisms does [Rust] fulfill this guarantee exactly?

"Fearless concurrency" is a Rust concept meaning you can write concurrent code as safely and easily as regular code, without fear of making subtle errors (data races, race conditions, deadlocks at the memory level). The compiler guarantees safety at compile time.

Mechanisms that ensure this:
Ownership and borrowing system:

Data can be passed to a thread only by following the rules: either by transferring ownership (move) or using safe synchronization primitives.

The compiler does not allow data to outlive its owner or to be mutably accessed from multiple threads simultaneously without synchronization.

Send and Sync traits:
Send — a type can be safely transferred to another thread.

Sync — a type can be safely referenced from multiple threads simultaneously (&T).

These traits are automatically implemented for most types, but if a type is not thread-safe (e.g., Rc<T>), it does not implement Send/Sync. Attempting to use such a type in another thread will cause a compilation error.

No data races at compile time:
A data race occurs when there is concurrent access to data where at least one access is a write and there is no synchronization.

Rust prohibits this through borrowing rules: you cannot have a mutable reference and any other reference to the same data simultaneously, even across different threads.

Standard synchronization primitives:
Types like Mutex<T>, RwLock<T>, Arc<T> (atomically reference-counted pointer) are designed so that they implement Send and Sync only under appropriate conditions.

For example, Arc<Mutex<T>> allows safely sharing mutable data between threads.

Channels (std::sync::mpsc):
Provide a safe way to pass messages between threads, following the principle "communicate via messages, not shared memory."

Conclusion: The Rust compiler prevents code that could lead to data races or incorrect memory access across multiple threads from compiling. Concurrency errors become compile-time errors rather than runtime failures.

- Why do [`Send`] and [`Sync`] exist at all? How is it related to interior mutability?

Send and Sync exist to provide a compile-time guarantee of thread safety. They are fundamental to Rust's approach to concurrency because they allow the compiler to enforce that data is used safely across threads.

Interior mutability refers to the ability to mutate data even when it is behind an immutable reference. Types like `RefCell<T>` and `Mutex<T>` enable this. However, they often aren't thread-safe by default because they can lead to data races if not properly synchronized.

The relationship between Send/Sync and interior mutability is that:

1. Types with interior mutability (like `RefCell<T>`) are typically not `Sync` because multiple threads accessing them through immutable references could lead to data races during mutation.
2. Types like `Mutex<T>` are both `Send` and `Sync`. This means they can be safely transferred between threads and shared across threads, which is essential for safe concurrent programming with interior mutability.
3. When implementing custom types with interior mutability, you must carefully consider whether they should be `Send` and/or `Sync` based on their internal structure and how they handle borrowing and mutation.

In essence, Send and Sync provide the framework within which interior mutability can be used safely in a multi-threaded context.



[`Send`]: https://doc.rust-lang.org/std/marker/trait.Send.html
[`Sync`]: https://doc.rust-lang.org/std/marker/trait.Sync.html
[Rust]: https://www.rust-lang.org

[1]: https://doc.rust-lang.org/nomicon/races.html
[2]: https://doc.rust-lang.org/book/ch16-00-concurrency.html
[3]: https://doc.rust-lang.org/book/ch16-04-extensible-concurrency-sync-and-send.html
[4]: https://doc.rust-lang.org/stable/nomicon/send-and-sync.html
[5]: http://huonw.github.io/blog/2015/02/some-notes-on-send-and-sync
[6]: https://nyanpasu64.github.io/blog/an-unsafe-tour-of-rust-s-send-and-sync
[7]: https://blog.reverberate.org/2021/12/18/thread-safety-cpp-rust.html
[8]: https://cliffle.com/blog/not-thread-safe
[9]: https://web.archive.org/web/20220929143451/https://itsallaboutthebit.com/arc-mutex
[10]: https://blog.dureuill.net/articles/too-dangerous-cpp
[11]: https://blog.cuongle.dev/p/this-sendsync-secret-separates-professional-and-amateur

Step 1.2: Boxing and pinning
============================

__Estimated time__: 1 day




## Boxing

[`Box`] is a pointer that owns heap-allocated data. This is the most common and simplest form of [heap] allocation in [Rust].

It's more idiomatic to use references (`&T`/`&mut T`) for pointing to the data, however they often come with lifetime complexity. [`Box`] allows to avoid this complexity at the cost of heap allocation.

[`Box`] is also a way to go if an owned [slice] is needed, but is not intended to be resized. For example, `Box<str>`/`Box<[T]>` are often used instead of `String`/`Vec<T>` in such cases.

To better understand [`Box`]'s purpose, design, limitations, and use cases, read through:
- [Rust Book: 15.1. Using Box to Point to Data on the Heap][1]
- [Official `std::boxed` docs][`std::boxed`]
- [Amos: What's in the box?][3]
- [Mahdi Dibaiee: What is `Box<str>` and how is it different from `String` in Rust?][8]




## Pinning

It is sometimes useful to have objects that are guaranteed to not move, in the sense that their placement in memory does not change, and can thus be relied upon. A prime example of such a scenario would be building self-referential structs, since moving an object with pointers to itself would invalidate them, which could cause undefined behavior.

[`Pin<P>`][`Pin`] ensures that the pointee of any pointer type `P` has a stable location in memory, meaning it cannot be moved elsewhere and its memory cannot be deallocated until it gets dropped. We say that the pointee is "pinned".

However, many types are always freely movable, even when pinned, because they do not rely on having a stable address. This includes all the basic types (like `bool`, `i32`, references) as well as types consisting solely of these types. Types that do not care about pinning implement the [`Unpin`] marker trait, which cancels the effect of [`Pin`]. For `T: Unpin`, `Pin<Box<T>>` and `Box<T>` function identically, as do `Pin<&mut T>` and `&mut T`.

Note, that pinning and [`Unpin`] only affect the pointed-to type `P::Target`, not the pointer type `P` itself that got wrapped in `Pin<P>`. For example, whether or not `Box<T>` is `Unpin` has no effect on the behavior of `Pin<Box<T>>` (here, `T` is the pointed-to type).

To better understand [`Pin`]'s purpose, design, limitations, and use cases, read through:
- [Official `std::pin` docs][`std::pin`]
- [Reddit: Pinned objects ELI5?][2]
- [SoByte: Pin and Unpin in Rust][10]
- [Adam Chalmers: Pin, Unpin, and why Rust needs them][4]
- [Tamme Schichler: Pinning in plain English][5]
- [Yoshua Wuyts: Safe Pin Projections Through View Types][6]
- [Official `#[pin_project]` docs][7]
- [Alice Ryhl answers on "Pin tutorial are confusing me"][9]
- [Rust Forum: Why is it unsafe to pin a shared reference?][11]
- [Ohad Ravid: Put a Pin on That][12]
- [Razieh Behjati: Leaky Abstractions and a Rusty Pin][13]
- [Saoirse Shipwreckt: Pin][14]




## Task

1. For the following types: `Box<T>`, `Rc<T>`, `Vec<T>`, `String`, `&[u8]`, `T`.  
   Implement the following traits:
   ```rust
   trait SayHi: fmt::Debug {
       fn say_hi(self: Pin<&Self>) {
           println!("Hi from {:?}", self)
       }
   }
   ```
   ```rust
   trait MutMeSomehow {
       fn mut_me_somehow(self: Pin<&mut Self>) {
           // Implementation must be meaningful, and
           // obviously call something requiring `&mut self`.
           // The point here is to practice dealing with
           // `Pin<&mut Self>` -> `&mut self` conversion
           // in different contexts, without introducing 
           // any `Unpin` trait bounds.
       }
   }
   ```

2. For the following structure:
   ```rust
   struct MeasurableFuture<Fut> {
       inner_future: Fut,
       started_at: Option<std::time::Instant>,
   }
   ```
   Provide a [`Future`] trait implementation, transparently polling the `inner_future`, and printing its execution time in nanoseconds once it's ready. Using `Fut: Unpin` trait bound (or similar) is not allowed. 




## Questions

After completing everything above, you should be able to answer (and understand why) the following questions:
- What does "boxing" mean in [Rust]? How is it useful? When and why is it required?

Boxing (Box<T>) allocates data on the heap and returns a pointer. Box owns the data.

How is it useful:
Enables recursive types (e.g., linked lists)

Provides heap allocation for large data

Allows moving unsized types (dyn Trait, [T])

Enables transferring ownership without copying

When required:
When type size is unknown at compile time

When you need stable address (though Pin is better for this)

- What is [`Pin`] and why is it required? What guarantees does it provide? How does it fulfill them?

Pin is a wrapper type that prevents the value inside from being moved in memory.

Why required: To safely work with self-referential structs and async generators—types where moving them in memory would invalidate internal pointers.

Pin provides two key guarantees:
For T: Unpin: No restrictions - the value can still be moved

For T: !Unpin: The value cannot be moved after being pinned

The guarantee is: once a value is pinned, it will stay at the same memory location until it's dropped


How does it fulfill them?
1.Via the Unpin marker trait:
Unpin is an autotrait that is automatically implemented for most types.

Types that should not move after being pinned explicitly implement !Unpin.

The compiler uses this information during type checking to allow or disallow certain operations.


2.Due to limitations in the Pin API:
The get_mut() method is only available for types implementing Unpin, allowing a mutable reference to be safely obtained only when it is safe to move.

The get_unchecked_mut() method requires unsafe for !Unpin types, shifting the responsibility for ensuring non-movability to the programmer.

Deref returns an immutable reference that never allows the value to be moved.

DerefMut is conditionally available only through Unpin.

Also through creation control and through pointer wrapping.

- How does [`Unpin`] affect the [`Pin`]? What does it mean?

Unpin is a marker that says, "This type is safe to move even after being pinned."

Pin is a wrapper around a pointer that promises not to move the value.

Without Unpin, there is no easy way to get &mut T out of Pin - the compiler forces the type to be handled through safe methods, preventing it from being moved.

- Is it allowed to move pinned data after the [`Pin`] dies? Why?
Yes, it is allowed. Once the Pin wrapper dies (is dropped), the pinned data can be moved again

Why?
Pin is just a compile-time contract, not a runtime invariant. It only guarantees immovability while the Pin exists.

Pin is a guard: It prevents moves only while it holds the pointer

Dropping is safe: When Pin drops, it either:

Drops the value (if it owns it, like Pin<Box<T>>)

Or releases the reference (if it's Pin<&mut T>)

After destruction: No restrictions remain — the value is either gone or back to normal Rust ownership rules

- What is structural pinning? When should it be used and why?
Structural pinning is a design pattern where a type propagates the "pinned" state to its fields. When the container is pinned, its structurally-pinned fields are also considered pinned.

In practice, this means:
The container implements Deref/DerefMut in a way that respects pinning

Methods that take Pin<&mut Self> can safely access pinned fields

Use When:
Your type contains self-referential fields

You're building abstractions over pinned data

You need to expose pinned access to fields while keeping the whole container pinned

Why should it be used
1. Safety Without Complexity
Without structural pinning, you'd need unsafe to access fields of pinned containers
2.Building Safe Abstractions
Structural pinning enables composable safe APIs
3.Projections Within Pinned Data
It allows creating projections — pinned references to fields


- What is [`Pin`] projection? Why does it exist? How is it used?
Pin projection is the process of moving from a Pin<&mut Container> to a Pin<&mut Field> — getting a pinned reference to a field of a pinned container.
It's the mechanism that enables structural pinning in practice.

Why does it exists:
When you have a pinned container, you can't just take a normal &mut Field — that would allow moving the field out of the container, breaking pinning guarantees

Pin projection provides a safe way to access fields while preserving pinning guarantees

How is it used:
```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct MyWrapper<F> {
    inner: F,
    ready: bool,
}

impl<F: Future> Future for MyWrapper<F> {
    type Output = F::Output;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Project to inner future
        let inner = unsafe { self.as_mut().map_unchecked_mut(|w| &mut w.inner) };
        
        // Poll the inner future
        match inner.poll(cx) {
            Poll::Ready(val) => {
                // Mark as ready and return
                unsafe { self.get_unchecked_mut().ready = true };
                Poll::Ready(val)
            }
            Poll::Pending => Poll::Pending
        }
    }
}
```


[`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
[`Future`]: https://doc.rust-lang.org/std/future/trait.Future.html
[`Pin`]: https://doc.rust-lang.org/std/pin/struct.Pin.html
[`std::boxed`]: https://doc.rust-lang.org/std/boxed/index.html
[`std::pin`]: https://doc.rust-lang.org/std/pin/index.html
[`Unpin`]: https://doc.rust-lang.org/std/marker/trait.Unpin.html
[heap]: https://en.wikipedia.org/wiki/Memory_management#HEAP
[Rust]: https://www.rust-lang.org
[slice]: https://doc.rust-lang.org/std/primitive.slice.html

[1]: https://doc.rust-lang.org/book/ch15-01-box.html
[2]: https://www.reddit.com/r/rust/comments/9akmqv/pinned_objects_eli5
[3]: https://fasterthanli.me/articles/whats-in-the-box
[4]: https://blog.adamchalmers.com/pin-unpin
[5]: https://blog.schichler.dev/pinning-in-plain-english-ckwdq3pd0065zwks10raohh85
[6]: https://blog.yoshuawuyts.com/safe-pin-projections-through-view-types
[7]: https://docs.rs/pin-project/latest/pin_project/attr.pin_project.html
[8]: https://web.archive.org/web/20230605135444/https://mahdi.blog/rust-box-str-vs-string
[9]: https://users.rust-lang.org/t/pin-tutorial-are-confusing-me/91003/18
[10]: https://www.sobyte.net/post/2022-07/rust-pin-unpin
[11]: https://users.rust-lang.org/t/why-is-it-unsafe-to-pin-a-shared-reference/40309
[12]: https://ohadravid.github.io/posts/2023-07-put-a-pin-on-that
[13]: https://itnext.io/leaky-abstractions-and-a-rusty-pin-fbf3b84eea1f
[14]: https://without.boats/blog/pin

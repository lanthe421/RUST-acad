Step 1.3: Shared ownership and interior mutability
==================================================

__Estimated time__: 1 day




## Shared ownership

[Rust] ownership model allows _only one owner of a value_. However, there are situations when multiple ownership is required, and it's important to understand how this can be accomplished.

The key piece is to put a value behind a smart pointer, so the pointer itself can be __cloned many times__ (thus allowing multiple owners), but is __pointing always to the same value__ (thus sharing a value). In [Rust] there is a [`Rc`] (["reference counted"][`std::rc`]) smart pointer for this purpose, and [`Arc`] ("atomic reference counted") for use in multiple threads. Both automatically destroy a value once there are no references left.

The code below won't compile as `a` is owned by `x` and moved to a heap before is passed to `y`:
```rust
struct Val(u8);

let a = Val(5);
let x = Box::new(a);
let y = Box::new(a);
```
```rust
error[E0382]: use of moved value: `a`
 --> src/main.rs:6:22
  |
5 |     let x = Box::new(a);
  |                      - value moved here
6 |     let y = Box::new(a);
  |                      ^ value used here after move
  |
  = note: move occurs because `a` has type `Val`, which does not implement the `Copy` trait
```

However, [`Rc`] allows that:
```rust
let a = Rc::new(Val(5));
let x = Rc::clone(&a);  // does not clone original value,
let y = Rc::clone(&a);  // but rather produces new reference to it
```

The [`Rc`], however, __should be used wisely__ as __won't deallocate memory on references cycle__ which is exactly what a __memory leak__ is. [Rust] is unable to prevent memory leaks at compile time (though makes hard to produce them). If it's still required to have a references cycle, you should use a [`Weak`] smart pointer ("weak reference") in combination with [`Rc`]. [`Weak`] allows to break a references cycle as can refer to a value that has been dropped already (returns `None` in such case). 

To better understand [`Rc`]/[`Weak`]'s purpose, design, limitations and use cases, read through:
- [Rust Book: 15.4. Rc, the Reference Counted Smart Pointer][1]
- [Rust Book: 15.6. Reference Cycles Can Leak Memory][2]
- [Official `std::rc` docs][`std::rc`]




## Interior mutability

[Rust] memory safety is based on the following rules (known as "borrowing rules"):

> Given an object `T`, it is only possible to have one of the following:
> - Having several immutable references (`&T`) to the object (also known as __aliasing__).
> - Having one mutable reference (`&mut T`) to the object (also known as __mutability__).

However, quite often there are situations where these rules are not flexible enough and it's required to have multiple references to a value and yet mutate it. [`Cell`] and [`RefCell`] __encapsulate mutability inside__ (thus called "interior mutability") and __provide interface which can be used through common shared references__ (`&T`). [`Mutex`] and [`RwLock`] serve the same purpose, but in a multi-threaded context.

These containers __allow to overcome [Rust] borrowing rules and track borrows at runtime__ (so called "dynamic borrowing"), which, obviously, leads to less safe code as compile-time errors become runtime panics. That's why one should __use [`Cell`]/[`RefCell`] wisely and only as a last resort__.

To better understand [`Cell`]/[`RefCell`]'s purpose, design, limitations and use cases, read through:
- [Rust Book: 15.5. RefCell and the Interior Mutability Pattern][3]
- [Official `std::cell` docs][`std::cell`]
- [Paul Dicker: Interior mutability patterns][6]
- [David Tolnay: Accurate mental model for Rust’s reference types][8]


### Advanced borrowing patterns

Notably, if the ownership over the value can be expressed separately from its data, the __interior mutability is possible while preserving compile-time borrowing checks and eliminating run-time overhead__, as proven by the [`qcell`] and [`ghost-cell`] crates.

To better understand their design, limitations and use cases, read through:
- [Official `qcell` crate docs][`qcell`]
- [Official `ghost-cell` crate docs][`ghost-cell`]
- [RustBelt: GhostCell: Separating Permissions from Data in Rust][9]




## Shared mutability

The most spread case is a combination of two previous: `Rc<RefCell<T>>` (or `Arc<Mutex<T>>`). This allows to mutate a value by multiple owners.

A real-world example would be a database client object: it _must be mutable_, as mutates its state under-the-hood (opens network connections, manages database sessions, etc), yet _we need to own it in multiple places_ of our code, not a single one.

The following articles may explain you this concept better:
- [Manish Goregaokar: Wrapper Types in Rust: Choosing Your Guarantees][4]
- [Alexandre Beslic: Rust, Builder Pattern, Trait Objects, `Box<T>` and `Rc<T>`][5]




## Avoiding panics and deadlocks

There is a simple rule for omitting deadlocks with [`Mutex`]/[`RwLock`] (applicable for panics with [`Cell`]/[`RefCell`] types too):

> Locking scopes must not intersect in any way.

The following example explains why deadlocks happen:
```rust
let owner1 = Arc::new(Mutex::new("string"));
let owner2 = owner1.clone();

let value = owner1.lock.unwrap();

// owner2 locking scope intersects with owner1 lock's scope.
let value = owner2.lock.unwrap(); 
```

Let's remove the intersection:
```rust
let owner1 = Arc::new(Mutex::new("string"));
let owner2 = owner1.clone();
{
    let value = owner1.lock.unwrap();
    // No intersection as owner1 locking scope ends here.
}
{
    let value = owner2.lock.unwrap();
}
```

That's why, usually, you should __omit to expose `Rc<RefCell<T>>`__ (or `Arc<Mutex<T>>`) __in API__'s, but rather __make them an inner implementation detail__. Doing this way you have full control over all locking scopes inside your methods (no scope can expand to outside), so __ensure that no intersection will happen__, and __expose a totally safe API__.

```rust
#[derive(Clone)]
struct SharedString(Arc<Mutex<String>>);

impl SharedString {
    fn mutate_somehow(&self) {
        let mut val = self.lock.unwrap();
        *val = "another string"
    }
}

let owner1 = SharedString(Arc::new(Mutex::new("string")));
let owner2 = owner1.clone();

// We are mutating the same value here,
// but no locking scopes intersection may happen by design.
// Such API will never deadlock or panic 
// due to runtime violation of borrowing rules.
owner1.mutate_somehow();
owner2.mutate_somehow();
```

And even when there is no possibility to hide lock guards behind API boundary, it may be feasible to try encoding the described property via type system, using zero-sized wrapper types on guards. See the following articles for examples and design insights:
- [Adrian Taylor: Can the Rust type system prevent deadlocks?][7]




## Task

Write a `GlobalStack<T>` collection which represents a trivial unsized [stack] (may grow infinitely) and has the following semantics:
- can be mutated through multiple shared references (`&GlobalStack<T>`);
- cloning doesn't clone data, but only produces a pointer, so multiple owners mutate the same data.




## Questions

After completing everything above, you should be able to answer (and understand why) the following questions:
- What is shared ownership? Which problem does it solve? Which penalties does it have?

In Rust, shared ownership refers to the ability for multiple parts of a program to own the same data simultaneously. This is a fundamental departure from Rust's default ownership model, where every value has a single, exclusive owner.

Rust enforces a strict rule: a value can have either one owner (exclusive access) or multiple owners (shared access), but not both at the same time unless the shared access is managed through specific, safe abstractions.

Shared ownership is not the default. It is implemented using smart pointers that allow a single value to be owned by multiple bindings. The primary tools for shared ownership in Rust are:

Rc<T> (Reference Counted): Enables shared ownership on the single-threaded heap.

Arc<T> (Atomically Reference Counted): Enables shared ownership on the multi-threaded heap. It uses atomic operations for thread safety.

Weak<T>: A non-owning reference used alongside Rc or Arc to prevent reference cycles and memory leaks.

These smart pointers work by storing the data on the heap and maintaining a reference count. When you clone an Rc or Arc, you don't clone the data; you increment the reference count and create a new pointer to the same heap-allocated data.


What problem does it solve?
It allows you to go beyond Rust's strict tree-based ownership. It's useful for:

Graph structures (not trees)

Cyclic relationships (paired with Weak)

Avoiding complex lifetimes ('a, 'b)

Safe access to data from multiple threads

Which penalties does it have:
Time overhead: reference counting (Arc's atomic counting is more expensive)

Memory overhead: heap data + control block (usually 16 bytes)

Code complexity: explicit .clone() everywhere, requires a RefCell/Mutex for modifications

Leak risk: cyclic references are not automatically deallocated, requiring the use of Weak


- What is interior mutability? Why is it required in [Rust]? In what price it comes?

Interior mutability is a design pattern in Rust that allows you to mutate data even when there are immutable references (&T) to it.

Normally, Rust's borrowing rules forbid mutation through an immutable reference. Interior mutability bypasses this by shifting borrow checking from compile time to runtime, using unsafe code internally wrapped in a safe API.

The primary types that provide interior mutability:

RefCell<T> — single-threaded, checks borrows at runtime (panics on violation)

Mutex<T> / RwLock<T> — thread-safe, blocks threads instead of panicking


Why Is It Required?
Interior mutability solves the problem: "How do I mutate data when the API forces me to have only &self?"

Classic scenario — shared ownership with Rc/Arc:
```rust
use std::rc::Rc;
use std::cell::RefCell;

let shared_data = Rc::new(RefCell::new(42));

let owner1 = Rc::clone(&shared_data);
let owner2 = Rc::clone(&shared_data);

*owner1.borrow_mut() = 100;  // Mutate through immutable Rc
println!("{}", owner2.borrow()); // 100
```
Without interior mutability, Rc<T> only gives &T — immutable access. You'd have no way to modify shared data. RefCell wraps the data and allows mutation through runtime checks


- Is it possible to write a custom type with interior mutability without using `std`? Why?

Yes, it's possible.
Why it's possible:

UnsafeCell is in the kernel, not in the standard code—it's a fundamental primitive that enables internal mutability through unsafe code.

Internal mutability doesn't require emitting systems—it's based on:
  Deadline checking (borrow counters)

  Atomic operations (for thread safety)

  All of this is available in the kernel, without any OS dependencies.

core provides all the necessary tools:

  UnsafeCell — the basis for any internal mutability

  Cell and RefCell (you can implement them yourself)

  Atomic types (AtomicUsize, AtomicPtr)

  Synchronization primitives based on atomic operations

The only limitation is that without std, you can't use high-level abstractions like Mutex or RwLock, but you can implement their equivalents yourself using atomic operations.

- What is shared mutability? Which are its common use-cases?

Shared mutability is the ability to mutate data through shared references (&T), bypassing Rust's usual rule that mutation requires exclusive access (&mut T).

Common Use-Cases:
Reference counting — Rc<T> and Arc<T> need to mutate internal reference counts while sharing ownership

Caching/memoization — lazily computing and storing values behind a shared reference

Observability — logging, metrics, or debug counters that need to be updated from multiple shared references

Self-referential structures — data structures where fields point to each other within the same type

Shared state in concurrent programs — thread-safe data structures like Arc<Mutex<T>> or Arc<RwLock<T>>

Mock objects in testing — tracking calls and behavior through shared test doubles

- How can we expose panic/deadlock-free API to users when using interior mutability?
Use Fallible Methods:
Return Result or Option instead of panicking


Avoid Recursive Locking:
Use RefCell instead of Mutex for single-threaded code (prevents deadlocks through runtime checks)

Use ReentrantMutex if reentrancy is actually needed

Document lock ordering requirements


Minimize Critical Sections:
Keep locks held for as short as possible


Handle Poisoning:
Use PoisonError to recover from panics in mutexes:
```rust
let guard = mutex.lock().unwrap_or_else(|poisoned| {
    poisoned.into_inner()  // Recover despite panic
});
```

Use Atomic Operations Where Possible:
Atomics are inherently deadlock-free and panic-free

Document Panic Conditions:
Clearly document when methods may panic (e.g., "Panics if already borrowed mutably")

Use Non-Panicking Alternatives:
try_borrow_mut() instead of borrow_mut()

compare_exchange() instead of store() with assertions

Fallible initialization patterns


Key Principle
Separate decision from action — validate before executing, return errors instead of panicking, and keep critical sections free of user-provided code that might panic.


[`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html
[`Cell`]: https://doc.rust-lang.org/std/cell/struct.Cell.html
[`ghost-cell`]: https://docs.rs/ghost-cell
[`Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
[`qcell`]: https://docs.rs/qcell
[`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html
[`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
[`RwLock`]: https://doc.rust-lang.org/std/sync/struct.RwLock.html
[`Weak`]: https://doc.rust-lang.org/std/rc/struct.Weak.html
[stack]: https://en.wikipedia.org/wiki/Stack_(abstract_data_type)
[`std::cell`]: https://doc.rust-lang.org/std/cell
[`std::rc`]: https://doc.rust-lang.org/std/rc
[Rust]: https://www.rust-lang.org

[1]: https://doc.rust-lang.org/book/ch15-04-rc.html
[2]: https://doc.rust-lang.org/book/ch15-06-reference-cycles.html
[3]: https://doc.rust-lang.org/book/ch15-05-interior-mutability.html
[4]: https://manishearth.github.io/blog/2015/05/27/wrapper-types-in-rust-choosing-your-guarantees
[5]: https://abronan.com/rust-trait-objects-box-and-rc
[6]: https://pitdicker.github.io/Interior-mutability-patterns
[7]: https://medium.com/@adetaylor/can-the-rust-type-system-prevent-deadlocks-9ae6e4123037
[8]: https://docs.rs/dtolnay/latest/dtolnay/macro._02__reference_types.html
[9]: https://plv.mpi-sws.org/rustbelt/ghostcell

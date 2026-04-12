Step 2.3: Bound behavior, not data
==================================

__Estimated time__: 1 day

Often, when we want to abstract over some type or behavior in [Rust] we are going from this:
```rust
struct UserService {
    repo: UserRepo,
}
```
to this:
```rust
struct UserService<R: UserRepo> {
    repo: R,
}
```
We specify `R: UserRepo` bound here as we want to restrict types in `repo` field to implement `UserRepo` behavior.

However, such restriction directly on a type leads to what is called "trait bounds pollution": we have to repeat this bound in every single `impl`, even in those ones, which has no relation to `UserRepo` behavior at all.
```rust
struct UserService<R: UserRepo> {
    repo: R,
}

impl<R> Display for UserService<R>
where
    R: Display + UserRepo, // <- We are not interested in UserRepo here,
{                          //    all we need is just Display.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserService with repo {}", self.repo)
    }
}
```
In a complex codebase such pollution multiplies from different types and may become a nightmare at some point.

The solution to this problem would be to understand that a __trait represents a certain behavior__, and, in reality, __we need that behavior only when we're declaring one__. Type declaration has nothing about behavior, it's all about _data_. __It's functions and methods where behavior happens__. So, let's just expect certain behavior when we really need this:
```rust
struct UserService<R> {
    repo: R,
}

// Expect Display when we expressing Display behavior.
impl<R: Display> Display for UserService<R> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UserService with repo {}", self.repo)
    }
}

// Expect UserRepo when we expressing actual UserService behavior,
// which deals with Users.
impl<R: UserRepo> UserService<R> {
    fn activate(&self, user: User) {
        // Changing User state in UserRepo...
    }
}
```

Placing trait bounds on `impl` blocks, methods and functions, rather than on types, _reduces the trait bounds pollution_, _lowers [coupling][1] of code parts_ and _makes generic code more clean, straightforward and ergonomic_.




## Lift unnecessary bounds

As a more general rule: __you should try to lift trait bounds as much as possible__ (especially in a library code), as it enlarges a variety of usages for a type.

Sometimes this requires to omit using `#[derive]` as this may impose unnecessary trait bound. For example:
```rust
#[derive(Clone)]
struct Loader<K, V> {
    state: Arc<Mutex<State<K, V>>>,
}

struct My;

let loader: Loader<My, My> = ..;
let copy = loader.clone(); // compile error as `My` doesn't impl `Clone`
```
This happens because `#[derive(Clone)]` applies `K: Clone` and `V: Clone` bounds in the derived code, despite the fact that they are not necessary at all, as [`Arc` always implements `Clone`][2] (also, consider `T: ?Sized` bound in the [linked implementation][2], which lifts implicit `T: Sized` bound, so allows to use `Arc::clone()` even for [unsized types][3] too).

By providing hand-baked implementation we are able to clone values of `Loader<My, My>` type without any problems:
```rust
struct Loader<K, V> {
    state: Arc<Mutex<State<K, V>>>,
}

// Manual implementation is used to omit applying unnecessary Clone bounds.
impl<K, V> Clone for Loader<K, V> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}

let loader: Loader<My, My> = ..;
let copy = loader.clone(); // it compiles now!
```




## Task

Refactor the code contained in [this step's crate](src/main.rs) to reduce trait bounds pollution as much as possible.




## Questions

After completing everything above, you should be able to answer (and understand why) the following questions:
- Which problems do trait bounds impose in [Rust] when are placed on a type definition?

Trait bounds on a type definition (e.g., ``struct S<T: Trait>``) create the following problems:
1.Forced constraint propagation—any code using this type must also satisfy the same bounds, even if they are not needed for a specific operation.

2.Loss of flexibility—the type cannot store values ​​that do not implement the specified trait, even if these methods are never called.

3.Signature bloat—bounds must be repeated in all impl blocks and functions working with this type, creating a "viral" effect.

4.Difficulty in composition—creating types with multiple parameters, where different methods require different combinations of traits, becomes more difficult.

5.Compilation performance issues—each type parameter with a bound increases the number of generated monomorphic copies.

The correct approach is to not specify bounds in the type definition, but to add them only at the impl block and function level.

- Why placing trait bounds on `impl` blocks is better?

Trait bounds on `impl` blocks are better because they:
1. Reduce trait bounds pollution—bounds are only specified where needed, not everywhere.
2. Lower coupling—types are less tied to specific traits, increasing modularity.
3. Improve ergonomics—generic code becomes cleaner and easier to use.
4. Allow more flexible usage—types can be used with a wider range of parameters.
5. Prevent unnecessary constraints—code doesn't force consumers to satisfy bounds they don't actually need.

- When cannot we do that and should use trait bounds on a type definition? When is it preferred?

When we __must__ place trait bounds on a type definition:
1. When the type's internal structure or behavior fundamentally depends on the trait's associated types or methods.
2. When the type needs to store values that must implement the trait (e.g., `Vec<T: Clone>`).
3. When the trait is required for the type's core functionality and not just for specific methods.
4. When the type is part of an API contract that requires specific behavior from its type parameters.

It's preferred when the trait is essential to the type's design and not just a convenience for method implementations.

- What are the problems with `std` derive macros regarding type parameters? How could they be solved?

Derive macros can impose unnecessary trait bounds on type parameters, leading to:
1. Trait bounds pollution—bounds are applied even when not needed.
2. Reduced flexibility—prevents usage with types that don't implement the derived traits.
3. Compilation errors—when trying to use the derived type with non-conforming types.

These problems can be solved by:
1. Providing manual implementations instead of relying on derives.
2. Using conditional compilation or feature flags to control which derives are used.
3. Creating custom derive macros that are more selective about which bounds they apply.
4. Using marker traits or phantom types to avoid imposing unwanted bounds.




[Rust]: https://www.rust-lang.org

[1]: https://en.wikipedia.org/wiki/Coupling_(computer_programming)
[2]: https://doc.rust-lang.org/stable/std/sync/struct.Arc.html#impl-Clone
[3]: ../../1_concepts/1_7_sized

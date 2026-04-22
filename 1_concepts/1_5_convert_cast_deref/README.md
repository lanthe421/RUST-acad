Step 1.5: Conversions, casting and dereferencing
================================================

__Estimated time__: 1 day

As [Rust] is a [strongly typed][1] language, all type conversions must be performed explicitly in the code. As [Rust] has a rich type system (programming logic and semantics are mostly expressed in types rather than in values), type conversions are inevitable in almost every single line of code. Fortunately, [Rust] offers [well-designed type conversion capabilities][`std::convert`], which are quite ergonomic, intuitive and are pleasant to use.




## Value-to-value conversion

Value-to-value conversion in [Rust] is done with [`From`] and [`Into`] mirrored traits (implementing the first one automatically implements another one). These traits provide __non-fallible conversion__.

If your conversion may fail, then you should use [`TryFrom`]/[`TryInto`] analogues, which __allow failing in a controlled way__.

```rust
let num: u32 = 5;
let big_num: u64 = num.into();
let small_num: u16 = big_num.try_into().expect("Value is too big");
```

Note, that __all these traits consume ownership__ of a passed value. However, they [can be implemented for references too][2] if you're treating a reference as a value.

To better understand [`From`]/[`Into`]'s and [`TryFrom`]/[`TryInto`]'s purpose, design, limitations and use cases, read through:
- [Rust By Example: 6.1. From and Into][8]
- [Official `From` docs][`From`]
- [Official `Into` docs][`Into`]
- [Official `TryFrom` docs][`TryFrom`]
- [Official `TryInto` docs][`TryInto`]




## Reference-to-reference conversion

Quite often you don't want to consume ownership of a value for conversion, but rather to refer it as another type. In such case [`AsRef`]/[`AsMut`] should be used. They allow to do a __cheap non-fallible reference-to-reference conversion__.

```rust
let string: String = "some text".into();
let bytes: &[u8] = string.as_ref();
```

[`AsRef`]/[`AsMut`] are commonly implemented for smart pointers to allow referring a data behind it via regular [Rust] references.

To better understand [`AsRef`]/[`AsMut`]'s purpose, design, limitations and use cases, read through:
- [Official `AsRef` docs][`AsRef`]
- [Official `AsMut` docs][`AsMut`]
- [Ricardo Martins: Convenient and idiomatic conversions in Rust][10]


### Difference from [`Borrow`]

Novices in [Rust] are often confused with the fact that [`AsRef`]/[`AsMut`] and [`Borrow`]/[`BorrowMut`] traits have the same signatures, because it may not be clear which trait to use or implement for their needs.

See [explanation in `Borrow` trait docs][`Borrow`]:

> Further, when providing implementations for additional traits, it needs to be considered whether they should behave identical to those of the underlying type as a consequence of acting as a representation of that underlying type. Generic code typically uses `Borrow<T>` when it relies on the identical behavior of these additional trait implementations. These traits will likely appear as additional trait bounds.
> 
> In particular `Eq`, `Ord` and `Hash` must be equivalent for borrowed and owned values: `x.borrow() == y.borrow()` should give the same result as `x == y`.
> 
> If generic code merely needs to work for all types that can provide a reference to related type `T`, it is often better to use `AsRef<T>` as more types can safely implement it.

And [another one in `AsRef` trait docs][`AsRef`]:

> - Unlike `AsRef`, `Borrow` has a blanket impl for any `T`, and can be used to accept either a reference or a value.
> - `Borrow` also requires that `Hash`, `Eq` and `Ord` for a borrowed value are equivalent to those of the owned value. For this reason, if you want to borrow only a single field of a struct you can implement `AsRef`, but not `Borrow`.

So, as a conclusion:
- [`AsRef`]/[`AsMut`] means that the implementor type may be represented as a reference to the implemented type. More like one type contains another one, or is just generally reference-convertible to the one.
- [`Borrow`]/[`BorrowMut`] means that the implementor type is equivalent to the implemented type in its semantics, differing only in how its data is stored. More like one type is just a pointer to another one.

For example, it's natural for an `UserEmail` type to implement `Borrow<str>`, so it may be easily consumed in the code accepting `&str` (converted to `&str`), as they're semantically equivalent regarding `Hash`, `Eq` and `Ord`. And it's good for some execution `Context` to implement `AsRef<dyn Repository>`, so it can be extracted and used where needed, without using the whole `Context`.

To better understand [`AsRef`]/[`Borrow`]'s difference, read through:
- [Anup Jadhav: AsRef vs Borrow trait (ft. ChatGPT)][12]


### Inner-to-outer conversion

[`AsRef`]/[`AsMut`] are able to do only outer-to-inner reference conversion, but obviously not the opposite.

```rust
struct Id(u8);

impl AsRef<u8> for Id {
    fn as_ref(&self) -> &u8 {
        &self.0
    }
}

impl AsRef<Id> for u8 {
    fn as_ref(&self) -> &Id {
        &Id(*self)
    }
}
```
```
error[E0515]: cannot return reference to temporary value
  --> src/lib.rs:11:9
   |
11 |         &Id(*self)
   |         ^---------
   |         ||
   |         |temporary value created here
   |         returns a reference to data owned by the current function
```

However, there is nothing wrong with such conversion as long as memory layout of the inner type is the same for the outer type.

```rust
#[repr(transparent)]
struct Id(u8);

impl AsRef<Id> for u8 {
    fn as_ref(&self) -> &Id {
        unsafe { mem::transmute(self) }
    }
}
```

That's exactly what [`ref-cast`] crate checks and does, without necessity of writing `unsafe` explicitly. See [crate's documentation][`ref-cast`] for more explanations.




## Dereferencing

[`Deref`]/[`DerefMut`] standard library trait __allows to implicitly coerce from a custom type to a reference__ when dereferencing (operator `*v`) is used. The most common example of this is using [`Box<T>`][`Box`] where `&T` is expected.

```rust
fn hello(name: &str) {
    println!("Hello, {}!", name);
}

let m = Box::new(String::from("Rust"));
hello(&m);
```

To better understand [`Deref`]'s purpose, design, limitations and use cases, read through:
- [Rust Book: 15.2. Treating Smart Pointers Like Regular References with the Deref Trait][3]
- [Official `Deref` docs][`Deref`]
- [Tim McNamara: Explaining Rust’s Deref trait][13]


### Incorrect usage

The implicit coercion that [Rust] implements for [`Deref`] is a sweet honey pot which may lead you to misuse of this feature.

The common temptation is to use [`Deref`] in a combination with [newtype pattern][4], so you can use your inner type via outer type without any explicit requirements. However, this is considered to be a bad practice, and [official `Deref` docs][`Deref`] clearly states:

> __`Deref` should only be implemented for smart pointers.__

The wider explanation of this bad practice is given in [this SO answer][5] and [`Deref` polymorphism anti-pattern][6] description.




## Casting

For casting between types the [`as` keyword][`as`] is used in [Rust].

```rust
fn average(values: &[f64]) -> f64 {
    let sum: f64 = sum(values);
    let size: f64 = len(values) as f64;
    sum / size
}
```

However, it supports only a [small, fixed set of transformations][7], and __is [not idiomatic][11] to use when other conversion possibilities are available__ (like [`From`], [`TryFrom`], [`AsRef`]).

See also:
- [Rust By Example: 5.1. Casting][9]
- [Rust Reference: 8.2.4. Type cast expressions][7]




## Task

Implement the following types:
1. `EmailString` - a type, which value can be only a valid email address string.
2. `Random<T>` - a smart pointer, which takes 3 values of the pointed-to type on creation and points to one of them randomly every time is used.

Provide conversion and `Deref` implementations for these types on your choice, to make their usage and interoperability with `std` types easy and ergonomic.




## Questions

After completing everything above, you should be able to answer (and understand why) the following questions:
- How value-to-value conversion is represented in [Rust]? What is relation between fallible and infallible one?

Rust implements conversions represented through the trait system, which are either error-free or possibly error-prone.

Core traits:
Error-free conversions:
From<T> and Into<T> — always succeed and do not return a result. If "From" is implemented, "To" is automatically available. These are used for conversions that are guaranteed to work, such as i32 to i64 or &str to String.

Possibly error-prone conversions:
TryFrom<T> and TryInto<T> — return Result<T, E> because the conversion may fail. A classic example is parsing strings into numbers: "42".parse() or i32::try_from("abc") will return an error.

The connection between them:
The main connection is that error-free conversions are a special case of error-free conversions, where an error never occurs. You can implement TryFrom with the Infallible error type (which cannot be created), meaning the conversion always succeeds.

However, From and TryFrom are independent traits. The presence of From does not automatically result in TryFrom, and vice versa. But if you have From, you can easily implement TryFrom as always succeeding.

- How reference-to-reference conversion is represented in [Rust]? How its traits differ? When and which one should be used?

The conversion of references in Rust is represented by two main traits: AsRef<T> and Borrow<T>, as well as their mutable versions AsMut<T> and BorrowMut<T>.

AsRef is intended for cheap, universal conversions between references. It imposes no additional semantic requirements — you're simply stating that a type can be represented as a reference to another type. For example, String implements AsRef<str> because a string can be borrowed as a slice. The same goes for PathBuf and Path, Vec<T> and [T]. AsRef is used to create flexible APIs when a function needs to accept many types that can be converted into the required reference.

Borrow is technically similar but has an important semantic difference: it requires that the conversion preserves the behavior of hashing, equality comparison, and ordering. The original type and the borrowed type must be indistinguishable with respect to these traits. This is critically important for collections like HashMap and HashSet, which use hashing and comparison for key lookup.

The main difference is that AsRef can be implemented for any types where a meaningful cheap conversion to a reference exists. For instance, String implements AsRef<[u8]> for accessing its bytes, but it does not implement Borrow<[u8]> because hashing a string and hashing its byte representation yield different results.

Borrow, on the other hand, is typically implemented only for cases where the type is a "smart pointer" or a wrapper that is completely transparent. Classic examples: String → str, Box<T> → T, Rc<T> → T. These type pairs behave identically with respect to hashing and comparison.

When to use AsRef: in functions that need flexibility in argument types. For example, a function working with paths should accept impl AsRef<Path> so that the caller can pass &str, String, or PathBuf. This is standard practice in Rust for creating convenient APIs without sacrificing performance.

When to use Borrow: when working with collections, especially when you need to look up values by keys that don't match the key type in the collection. If you have a HashMap<String, Value>, you can look up by &str precisely because String implements Borrow<str>. You should also implement Borrow for your own types if they are transparent wrappers and will be used as keys in hash tables.

AsMut and BorrowMut are the mutable versions of these traits, following the same principles but returning mutable references.


- How can inner-to-outer reference conversion be achieved in [Rust]? Which prerequisites does it have?

How can inner-to-outer reference conversion be achieved in [Rust]?
By using self-referential structures with stable addresses, typically implemented via:
Pinning (Pin<Box<T>> or Pin<&mut T>) to prevent moving.

Raw pointers (*const/*mut) to store the inner-to-outer reference.

Safe wrappers like ouroboros, self_cell, or rental crates.

Which prerequisites does it have?
The value must be pinned (immovable after initialization).

The inner reference must be valid for the lifetime of the outer struct.

Construction must ensure the reference is initialized only after the outer struct’s address is stable.

Typically requires unsafe code or a safe abstraction crate to uphold Rust’s aliasing and lifetime rules.

- What is dereferencing in [Rust]? How it can be abused? Why it shouldn't be abused?

Dereferencing in Rust is the process of accessing the value that a reference points to using the * operator. It allows you to work with the actual data rather than a reference to it.

How it can be abused?
By implementing Deref for non-pointer types or types that aren't meant to be transparent wrappers, leading to confusing APIs and unexpected behavior.
Using Deref to automatically convert types in ways that aren't obvious to users of the API.
Creating complex, hard-to-debug self-referential structures that violate Rust's ownership guarantees.
Overusing Deref in newtype patterns to hide the underlying type too aggressively, making code less explicit and harder to reason about.

Why it shouldn't be abused?
Because it can make code less clear and introduce subtle bugs.
Because it's meant to be used primarily for smart pointers to provide transparent access to the contained value.
Because improper use can break Rust's ownership model and make code harder to maintain.

- Why using [`as`] keyword is not a good practice in [Rust]? Why do we still use it?

Why using [`as`] keyword is not a good practice in [Rust]?
Because it's limited to a small, fixed set of primitive type conversions and lacks the expressiveness and safety of dedicated conversion traits like From, Into, TryFrom, and TryInto.

Why do we still use it?
We still use it for:
- Explicit casting between primitive types (e.g., f64 to i32).
- Converting between integer types of different sizes (e.g., u32 to u16).
- Casting between raw pointers and integers.
- Situations where the compiler cannot infer the target type.
- Interoperability with C code or FFI.
- When the conversion is truly simple and safe, and no other trait applies.



[`as`]: https://doc.rust-lang.org/std/keyword.as.html
[`AsMut`]: https://doc.rust-lang.org/std/convert/trait.AsMut.html
[`AsRef`]: https://doc.rust-lang.org/std/convert/trait.AsRef.html
[`Borrow`]: https://doc.rust-lang.org/std/borrow/trait.Borrow.html
[`BorrowMut`]: https://doc.rust-lang.org/std/borrow/trait.BorrowMut.html
[`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html
[`Deref`]: https://doc.rust-lang.org/std/ops/trait.Deref.html
[`DerefMut`]: https://doc.rust-lang.org/std/ops/trait.DerefMut.html
[`From`]: https://doc.rust-lang.org/std/convert/trait.From.html
[`Into`]: https://doc.rust-lang.org/std/convert/trait.Into.html
[Rust]: https://www.rust-lang.org
[`ref-cast`]: https://docs.rs/ref-cast
[`std::convert`]: https://doc.rust-lang.org/std/convert/index.html
[`TryFrom`]: https://doc.rust-lang.org/std/convert/trait.TryFrom.html
[`TryInto`]: https://doc.rust-lang.org/std/convert/trait.TryInto.html

[1]: https://en.wikipedia.org/wiki/Strong_and_weak_typing
[2]: https://doc.rust-lang.org/std/string/struct.String.html#impl-From%3C%26%27_%20str%3E
[3]: https://doc.rust-lang.org/book/ch15-02-deref.html
[4]: https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html
[5]: https://stackoverflow.com/questions/45086595/is-it-considered-a-bad-practice-to-implement-deref-for-newtypes
[6]: https://rust-unofficial.github.io/patterns/anti_patterns/deref.html
[7]: https://doc.rust-lang.org/reference/expressions/operator-expr.html#type-cast-expressions
[8]: https://doc.rust-lang.org/rust-by-example/conversion/from_into.html
[9]: https://doc.rust-lang.org/rust-by-example/types/cast.html
[10]: https://ricardomartins.cc/2016/08/03/convenient_and_idiomatic_conversions_in_rust
[11]: https://rust-lang.github.io/rust-clippy/master/index.html#as_conversions
[12]: https://web.archive.org/web/20240220233335/https://rusty-ferris.pages.dev/blog/asref-vs-borrow-trait
[13]: https://timclicks.dev/article/explaining-rusts-deref-trait

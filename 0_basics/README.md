Step 0: Become familiar with Rust basics
========================================

__Estimated time__: 3 days

Read through [the Rust Book][Rust Book], [Rust FAQ], and become familiar with basic [Rust] concepts, syntax, the memory model, and the type and module systems.

Polish your familiarity by completing [Rust By Example] and [Rustlings][rustlings].

Read through [the Cargo Book][Cargo Book] and become familiar with [Cargo] and its workspaces.

After completing these steps, you should be able to answer (and understand why) the following questions:
- What memory model does [Rust] have? Is it single-threaded or multiple-threaded? Is it synchronous or asynchronous?
Rust has a memory ownership model, a borrowing system, and lifetimes.
There are both single-threaded simple programs and multi-threaded ones (in most cases).
Rust is synchronous, but it has libraries that add asynchrony to it as well (e.g. Tokio).

- What runtime does [Rust] have? Does it use a GC (garbage collector)?
Rust has a minimal runtime environment. Unlike Java (JVM) or Go (its own runtime with scheduler), Rust compiles directly to native code.
RUst doesn't have a garbage collector.
- What does static typing mean? What is a benefit of using it?
Static typing means that the type of each variable and expression is known at compile time, not runtime.

Advantages:

Safety: Most type-related errors (such as attempting to add a number to a string) are caught at compile time rather than failing in production.

Performance: The compiler knows the exact size of the data and can generate more optimized machine code. There's no need to spend time checking types at runtime.

Refactoring and Maintenance: Code is easier to understand and modify because types act as documentation. The IDE can provide precise hints (autocomplete).

- What are generics and parametric polymorphism? Which problems do they solve?

Generics are a way to write code that can work with different types without being tied to a specific one. They are an implementation of parametric polymorphism (code works with any type that satisfies certain conditions).

What problems do they solve?

De-duplication: Don't need to write separate functions for Vec<i32>, Vec<f64>, and Vec<String>.

Type safety: Unlike generics in some languages ​​(or using void* in C), Rust preserves type information. If you create a Vec<i32>, you can't put a string in it—the compiler will check for that.

- What are traits? How are they used? How do they compare to interfaces? What are auto traits and blanket impls? What is a marker trait?
Traits are Rust's way of defining shared behavior across types. They're similar to interfaces in other languages but with some unique features.
How are they used:
Defining shared behavior: Types that implement a trait can be used interchangeably

As bounds on generics: Restricting generic types to those with certain capabilities

For extension methods: Adding methods to existing types

How do they compare to interfaces?
Similarities:

Both define a contract of methods that types must implement

Both support polymorphism

Key differences:

Rust traits can have default implementations

Traits can be implemented for any type, even external ones (with some restrictions)

Traits support associated types and constants

Rust has orphan rules that prevent certain trait implementations

Traits can be used for both static and dynamic dispatch

Auto traits are traits that are automatically implemented for a type if all its components implement the trait. The most common examples are Send, Sync, Unpin, and Sized.

Blanket implementations are implementations of a trait for a wide range of types using generics.

Marker traits have no methods. They're used to mark types as having certain properties or capabilities.
- What are static and dynamic dispatch? Which should you use, and when?

These are two ways of calling methods when the code uses generics or traits.

Static Dispatch:

How it works: Used with generics (fn foo<T: Trait>(x: T)). The compiler creates (monomorphizes) a separate copy of the function for each concrete type you pass in. The call x.method() is replaced with a call to the specific function for that type.

Advantages: Maximum performance (calls are known at compile time, allowing for code inlining).

Disadvantages: Increases the binary size (due to code duplication for each type).

Dynamic Dispatch:

How it works: Used with trait objects (dyn Trait). A pointer to the data and a pointer to a virtual table (vtable) are created. The vtable contains the addresses of the methods for the specific type. The call happens through dereferencing the pointer at runtime.

Advantages: The same code can work with different types that are unknown at compile time (e.g., a heterogeneous list of shapes in an array). Results in a smaller code size (one function for all types).

Disadvantages: Slight overhead (indirect call through the vtable), and the compiler cannot inline the code.

Which one to use and when?

By default: Use Static Dispatch. It's faster and is used in most cases, especially in libraries where the types are known at compile time by the user.

Use Dynamic Dispatch when:

You need a heterogeneous collection (e.g., Vec<Box<dyn Display>>) that stores different types (numbers, strings) unified by common behavior.
It significantly reduces compilation time or code size if there are many generics.
You are returning a closure from a function (since each closure has its own unique type, it needs to be boxed as Box<dyn Fn>).  

- What is a crate and what is a module in [Rust]? How do they differ? How are they used?
What is a Crate?
A crate is the smallest compilation unit in Rust (the code the compiler processes at one time)

Can be either a binary crate (compiles to an executable, has main) or a library crate (shared code, no main)

Has a crate root (src/main.rs for binary, src/lib.rs for library)

Can be published and shared via crates.io

Each crate has a name and version defined in Cargo.toml

What is a Module?
A module is a way to organize code within a single crate

Controls visibility (privacy) of items using pub keyword

Creates a hierarchical namespace (mod math { ... })

Can be defined inline or in separate files

Key Differences
Aspect	        Crate	                    Module
Scope	        Independent package	        Inside one crate
Compilation	    Separate unit	            Part of its crate
Distribution	Can be published	        Cannot be published alone
Dependencies	Can depend on other crates	Uses parent crate's dependencies
How They're Used
Crates organize code at the distribution level (what others can import)

Modules organize code at the implementation level (how you structure your code internally)

use imports items (from crates or modules)

pub makes items visible outside their module

mod declares a module
- What are move semantics? What are borrowing rules? What is the benefit of using them?
- What is immutability? What is the benefit of using it?
- What is cloning? What is copying? How do they compare?
- What is RAII? How is it implemented in [Rust]? What is the benefit of using it?
- What is an iterator? What is a collection? How do they differ? How are they used?
- What are macros? Which problems do they solve? What is the difference between declarative and procedural macros?
- How is code tested in [Rust]? Where should you put tests and why?
- Why does [Rust] have `&str` and `String` types? How do they differ? When should you use them?
- What are lifetimes? Which problems do they solve? Which benefits do they give?
- Is [Rust] an OOP language? Is it possible to use SOLID/GRASP? Does it have inheritance?

_Additional_ articles, which may help to understand the above topic better:
- [George He: Thinking in Rust: Ownership, Access, and Memory Safety][19]
- [Chris Morgan: Rust ownership, the hard way][1]
- [Adolfo Ochagavía: You are holding it wrong][12]
- [Vikram Fugro: Beyond Pointers: How Rust outshines C++ with its Borrow Checker][15]
- [Sabrina Jewson: Why the “Null” Lifetime Does Not Exist][16]
- [HashRust: A guide to closures in Rust][13]
- [Ludwig Stecher: Rusts Module System Explained][2]
- [Tristan Hume: Models of Generics and Metaprogramming: Go, Rust, Swift, D and More][3]
- [Jeff Anderson: Generics Demystified Part 1][4]
- [Jeff Anderson: Generics Demystified Part 2][5]
- [Bradford Hovinen: Demystifying trait generics in Rust][14]
- [Brandon Smith: Three Kinds of Polymorphism in Rust][6]
- [Jeremy Steward: C++ & Rust: Generics and Specialization][7]
- [Lukasz Uszko: Safe and Secure Coding in Rust: A Comparative Analysis of Rust and C/C++][18]
- [cooscoos: &stress about &Strings][8]
- [Jimmy Hartzell: RAII: Compile-Time Memory Management in C++ and Rust][9]
- [Georgios Antonopoulos: Rust vs Common C++ Bugs][10]
- [Yurii Shymon: True Observer Pattern with Unsubscribe mechanism using Rust][11]
- [Clayton Ramsey: I built a garbage collector for a language that doesn't need one][17]




[Cargo]: https://github.com/rust-lang/cargo
[Cargo Book]: https://doc.rust-lang.org/cargo
[Rust]: https://www.rust-lang.org
[Rust Book]: https://doc.rust-lang.org/book
[Rust By Example]: https://doc.rust-lang.org/rust-by-example
[Rust FAQ]: https://prev.rust-lang.org/faq.html
[rustlings]: https://rustlings.cool

[1]: https://chrismorgan.info/blog/rust-ownership-the-hard-way
[2]: https://aloso.github.io/2021/03/28/module-system.html
[3]: https://thume.ca/2019/07/14/a-tour-of-metaprogramming-models-for-generics
[4]: https://web.archive.org/web/20220525213911/http://jeffa.io/rust_guide_generics_demystified_part_1
[5]: https://web.archive.org/web/20220328114028/https://jeffa.io/rust_guide_generics_demystified_part_2
[6]: https://www.brandons.me/blog/polymorphism-in-rust
[7]: https://www.tangramvision.com/blog/c-rust-generics-and-specialization#substitution-ordering--failures
[8]: https://cooscoos.github.io/blog/stress-about-strings
[9]: https://www.thecodedmessage.com/posts/raii
[10]: https://geo-ant.github.io/blog/2022/common-cpp-errors-vs-rust
[11]: https://web.archive.org/web/20230319015854/https://ybnesm.github.io/blah/articles/true-observer-pattern-rust
[12]: https://ochagavia.nl/blog/you-are-holding-it-wrong
[13]: https://hashrust.com/blog/a-guide-to-closures-in-rust
[14]: https://gruebelinchen.wordpress.com/2023/06/06/demystifying-trait-generics-in-rust
[15]: https://dev.to/vikram2784/beyond-pointers-how-rust-outshines-c-with-its-borrow-checker-1mad
[16]: https://sabrinajewson.org/blog/null-lifetime
[17]: https://claytonwramsey.github.io/2023/08/14/dumpster.html
[18]: https://luk6xff.github.io/other/safe_secure_rust_book/intro/index.html
[19]: https://cocoindex.io/blogs/rust-ownership-access

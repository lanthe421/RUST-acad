Step 2: Idioms
==============

__Estimated time__: 2 days

These steps describe common idioms required for writing well-designed and idiomatic [Rust] code.

> ❗️Before completing this step you should complete all its sub-steps.

After doing them you should be able to answer the following questions:
- Why should I care about types and expressing things in types? How do types help to increase guarantees of a program being correct?

Types are contracts enforced by the compiler, not by you at runtime. If a function takes a Coin instead of a raw u32, you simply cannot pass a product price there by mistake — the compiler rejects it. The more logic you encode in types, the fewer "technically possible but should never exist" states your program has. In our code, the Coin enum with fixed variants makes it impossible to construct a coin with value 3 or 7.

- What is essential for writing well-designed and ergonomic APIs in [Rust] and why?

Three things:

abstracting over input types (impl Into<String>, impl IntoIterator) — callers don't think about conversions

returning concrete types — callers get maximum information about what they received

typed errors via ``Result<T, E>`` — errors must be handled explicitly, they can't be silently ignored

- Why `mem::replace` exists and what purpose does it solve? When and why is it really helpful?

Rust doesn't allow moving a value out of a struct field while leaving it empty — that would violate ownership guarantees. mem::replace solves this by atomically swapping the field with a new value and returning the old one. It's useful when you need to take ownership of a field without destroying the struct — for example, replacing a Vec with an empty vec![] to take the original data out.

- How input type polymorphism is usually organized in [Rust] APIs? What cost does it have?

Through conversion traits: ``Into<T>``, ``AsRef<T>``, IntoIterator, etc. The compiler generates a separate version of the function for each concrete type — this is called monomorphization. The cost is binary size bloat. The common optimization is a thin generic wrapper that immediately converts and delegates to a single concrete implementation.

- Which ways and tools do exist for future-proofing source code in [Rust]?## More reading

``#[non_exhaustive]`` on enums/structs — external code can't exhaustively match all variants, so you can add new ones without a breaking change
sealed traits — the trait can't be implemented outside your crate, so you can evolve it freely
``..`` in struct patterns — ignores unknown fields in destructuring
hiding implementation details behind pub(crate) and modules — only expose what you commit to





## More reading

- [Matthias Endler: Idiomatic Rust: Patterns for Defensive Programming in Rust][11]
- [Rust Design Patterns]
- [Learning Material for Idiomatic Rust]




## Task

Design and implement a `VendingMachine` type, which behaves like a [vending machine][1]:
- `Product` should have a price and a name;
- `VendingMachine` should have a limited capacity of `Product`s;
- `VendingMachine` should be able to give change;
- `VendingMachine` should reject purchase if it cannot give change;
- `Coin` nominal values could only be `1`, `2`, `5`, `10`, `20` and `50`.

Make its usage API as convenient as you're capable to.




[Learning Material for Idiomatic Rust]: https://corrode.dev/blog/idiomatic-rust-resources
[Rust]: https://www.rust-lang.org
[Rust Design Patterns]: https://rust-unofficial.github.io/patterns

[1]: https://en.wikipedia.org/wiki/Vending_machine
[11]: https://corrode.dev/blog/defensive-programming

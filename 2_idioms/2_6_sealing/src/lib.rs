pub mod my_error;
pub mod my_iterator_ext;

pub use self::{my_error::MyError, my_iterator_ext::MyIteratorExt};

// ============================================================
// Proof: MyIteratorExt is sealed at the MODULE level.
//
// Uncommenting the code below will NOT compile, because
// `my_iterator_ext::private::Sealed` is not accessible here.
//
// struct MyIter;
// impl Iterator for MyIter {
//     type Item = i32;
//     fn next(&mut self) -> Option<i32> { None }
// }
// impl MyIteratorExt for MyIter {}
// ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ERROR: the trait `private::Sealed`
//                                is not implemented for `MyIter`
// ============================================================

// ============================================================
// Proof: MyError::type_id is sealed at the MODULE level.
//
// Uncommenting the code below will NOT compile, because
// `my_error::private::Token` is not accessible here.
//
// use std::any::TypeId;
// use std::fmt;
// #[derive(Debug)]
// struct MyErr;
// impl fmt::Display for MyErr {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "err") }
// }
// impl MyError for MyErr {
//     fn type_id(&self, _: ???) -> TypeId { TypeId::of::<u8>() }
//     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ ERROR: `my_error::private::Token`
//                                          is private and cannot be named
// }
// ============================================================

/// Proof: `MyIteratorExt` cannot be implemented outside this crate.
///
/// ```compile_fail
/// use step_2_6::MyIteratorExt;
///
/// struct MyIter;
/// impl Iterator for MyIter {
///     type Item = i32;
///     fn next(&mut self) -> Option<i32> { None }
/// }
///
/// // ERROR: `step_2_6::my_iterator_ext::private::Sealed` is not accessible
/// impl MyIteratorExt for MyIter {}
/// ```
pub fn _doctest_my_iterator_ext_sealed() {}

/// Proof: `MyError::type_id` cannot be overridden outside this crate.
///
/// ```compile_fail
/// use std::any::TypeId;
/// use std::fmt;
/// use step_2_6::MyError;
///
/// #[derive(Debug)]
/// struct MyErr;
///
/// impl fmt::Display for MyErr {
///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///         write!(f, "err")
///     }
/// }
///
/// impl MyError for MyErr {
///     // ERROR: `step_2_6::my_error::private::Token` is private
///     fn type_id(&self, _: step_2_6::my_error::private::Token) -> TypeId {
///         TypeId::of::<u8>()
///     }
/// }
/// ```
pub fn _doctest_my_error_type_id_sealed() {}

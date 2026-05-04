use std::collections::BTreeMap;

use step_3_2::btreemap;


// DECLARATIVE MACRO (also works)

// #[macro_export]
// macro_rules! BTreeMap {
//     ($(($k:expr, $v:expr)),*) => {
//         {
//             let mut temp_btreemap = BTreeMap::new();
//             $(
//                 temp_btreemap.insert($k, $v);
//             )*
//             temp_btreemap
//         }
//     };
// }

fn main() {
    let map = btreemap![("dsadas", "fdsas")];
    println!("{:?}", map);
}
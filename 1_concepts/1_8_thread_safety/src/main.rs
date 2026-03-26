use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex, MutexGuard};

// 1. Sync but !Send
//    MutexGuard is Sync (shared ref is ok) but !Send (can't move to another thread)
struct OnlySync {
    _guard: MutexGuard<'static, i32>,
}

// 2. Send but !Sync
//    Cell<T> is Send (can move to another thread) but !Sync (no shared refs across threads)
struct OnlySend {
    _cell: Cell<i32>,
}

// 3. Send + Sync
//    Arc<Mutex<T>> is both Send and Sync
struct SyncAndSend {
    _data: Arc<Mutex<i32>>,
}

// 4. !Send + !Sync
//    Rc<T> is neither Send nor Sync
struct NotSyncNotSend {
    _rc: Rc<i32>,
}

// Compile-time assertions
fn assert_sync<T: Sync>() {}
fn assert_send<T: Send>() {}
fn assert_not_sync<T: Sync>() {} // intentionally NOT called for !Sync types
fn assert_not_send<T: Send>() {} // intentionally NOT called for !Send types

fn main() {
    // Verify Send+Sync at compile time
    assert_sync::<OnlySync>();
    assert_send::<OnlySend>();
    assert_sync::<SyncAndSend>();
    assert_send::<SyncAndSend>();

    // These would fail to compile (uncomment to verify):
    // assert_send::<OnlySync>();       // error: OnlySync is !Send
    // assert_sync::<OnlySend>();       // error: OnlySend is !Sync
    // assert_send::<NotSyncNotSend>(); // error: NotSyncNotSend is !Send
    // assert_sync::<NotSyncNotSend>(); // error: NotSyncNotSend is !Sync

    // Play with SyncAndSend from multiple threads
    let shared = SyncAndSend { _data: Arc::new(Mutex::new(0)) };
    let clone = SyncAndSend { _data: Arc::clone(&shared._data) };

    let handle = std::thread::spawn(move || {
        let mut val = clone._data.lock().unwrap();
        *val += 1;
    });

    handle.join().unwrap();
    println!("value = {}", shared._data.lock().unwrap());

    // OnlySend can be moved to another thread
    let only_send = OnlySend { _cell: Cell::new(42) };
    let handle = std::thread::spawn(move || {
        println!("OnlySend value = {}", only_send._cell.get());
    });
    handle.join().unwrap();
}






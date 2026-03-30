use std::{
    fmt,
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::Instant,
};
use pin_project::pin_project;

trait SayHi: fmt::Debug {
    fn say_hi(self: Pin<&Self>) {
        println!("Hi from {:?}", self)
    }
}

trait MutMeSomehow {
    fn mut_me_somehow(self: Pin<&mut Self>);
}

// One blanket impl covers all: Box<T>, Rc<T>, Vec<T>, String, &[u8], T.

impl<T: fmt::Debug> SayHi for T {}

impl<T: Unpin> MutMeSomehow for T {
    fn mut_me_somehow(self: Pin<&mut Self>) {
        // Safe: T: Unpin means Pin<&mut T> == &mut T.
        let this: &mut T = Pin::into_inner(self);
        let _ = this; // &mut access obtained — point of the exercise
    }
}


#[pin_project]
struct MeasurableFuture<Fut> {
    #[pin]
    inner_future: Fut,
    started_at: Option<Instant>,
}



impl<Fut: Future> Future for MeasurableFuture<Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if this.started_at.is_none() {
            *this.started_at = Some(Instant::now());
        }
        match this.inner_future.poll(cx) { 
            Poll::Ready(output) => {
                let elapsed = this.started_at.unwrap().elapsed();
                println!("Future completed in {} ns", elapsed.as_nanos());
                Poll::Ready(output)
            }
            Poll::Pending => Poll::Pending,
         }
    }
}


fn main() {
    // SayHi
    Pin::new(&Box::new(42_i32)).say_hi();
    Pin::new(&Rc::new(7_i32)).say_hi();
    Pin::new(&vec![1_i32, 2, 3]).say_hi();
    Pin::new(&String::from("hello")).say_hi();
    Pin::new(&&[1_u8, 2, 3][..]).say_hi();
    Pin::new(&42_i32).say_hi();

    // MutMeSomehow
    let mut s = String::from("hello");

    Pin::new(&mut s).mut_me_somehow();

    let mut v: Vec<i32> = vec![1, 2, 3];
    Pin::new(&mut v).mut_me_somehow();

    let mut b: Box<i32> = Box::new(10);
    Pin::new(&mut b).mut_me_somehow();

    let mut rc: Rc<i32> = Rc::new(5);
    Pin::new(&mut rc).mut_me_somehow();

    let mut bytes: &[u8] = &[1, 2, 3];
    Pin::new(&mut bytes).mut_me_somehow();

    let mut x: i32 = 99;
    Pin::new(&mut x).mut_me_somehow();

    // MeasurableFuture 
    use std::task::{RawWaker, RawWakerVTable, Waker};

    fn noop_clone(p: *const ()) -> RawWaker {
        RawWaker::new(p, &NOOP_VTABLE)
    }
    fn noop(_: *const ()) {}
    static NOOP_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);

    let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &NOOP_VTABLE)) };
    let mut cx = Context::from_waker(&waker);

    let fut = MeasurableFuture {
        inner_future: std::future::ready("Hello"),
        started_at: None,
    };
    let mut pinned = std::pin::pin!(fut);
    if let Poll::Ready(v) = pinned.as_mut().poll(&mut cx) {
        println!("Got: {v}");
    }
}

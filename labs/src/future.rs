use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;
use smol::{io};
use async_io::Timer;

pub fn create_timer() -> impl Future<Output = i32> {
    async {
        println!("Creating timer");
        Timer::after(Duration::from_secs(1)).await;
        1
    }
}

pub fn ext_create_timer(a: bool) -> Pin<Box<dyn Future<Output=Box<dyn Any>>>> {
    let a = false; //Box::into_pin(1);

    if a {
        Box::pin(async {
            println!("Creating timer");
            Timer::after(Duration::from_secs(1)).await;
            Box::new(1) as Box<dyn Any>
        })
    } else {
        Box::pin(async {
            println!("Creating timer");
            Timer::after(Duration::from_secs(1)).await;
            Box::new("ready".to_string()) as Box<dyn Any>
        })
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::future::Future;
    use std::pin::Pin;
    use std::task::{Context, Poll};
    use crate::future::{create_timer, ext_create_timer};

    #[test]
    fn test_create_timer() {
        smol::block_on(async {
            let r  = create_timer().await;
            assert_eq!(r, 1);
        });
    }

    fn test_ext_create_timer() {
        smol::block_on(async {
            let r  = ext_create_timer(true).await;
            if let Some(v) = r.downcast_ref::<i8>() {
                println!("i8 {}", v)
            } else if let Some(v) = r.downcast_ref::<String>() {
                println!("string {}", v)
            }
        });

    }



    struct TracedFuture<T> {
        wrapped: Pin<Box<dyn Future<Output=T>>>,
        name: &'static str,
    }

    impl<T> TracedFuture<T> {
        fn new(wrapped: Pin<Box<dyn Future<Output=T>>>, name: &'static str) -> TracedFuture<T> {
            TracedFuture { wrapped, name }
        }
    }

    impl<T> Future for TracedFuture<T> {
        type Output = T;

        fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
            let name = self.name;
            println!("future '{:?}' poll start", name);

            let this = unsafe { self.get_unchecked_mut() };
            let result = unsafe { Pin::new_unchecked(&mut this.wrapped) }.poll(_cx);


            match result {
                Poll::Pending => println!("future '{:?}' poll pending", name),
                Poll::Ready(ref result) => {
                    println!("future '{:?}' poll ready", name)
                },
            }

            result
        }
    }

    #[test]
    fn test_traced_future() {
        smol::block_on(async {
            let timer_future  = create_timer();
            assert_eq!(TracedFuture::new(Box::pin(timer_future), "timer").await, 1);
        });
    }
}
use std::any::Any;
use std::future::Future;
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

pub fn ext_create_timer(a: bool) -> Box<dyn Future<Output=Box<dyn Any>>> {
    let a = false; //Box::into_pin(1);

    if a {
        Box::new(async {
            println!("Creating timer");
            Timer::after(Duration::from_secs(1)).await;
            Box::new(1) as Box<dyn Any>
        })
    } else {
        Box::new(async {
            println!("Creating timer");
            Timer::after(Duration::from_secs(1)).await;
            Box::new("ready".to_string()) as Box<dyn Any>
        })
    }
}

#[cfg(test)]
mod tests {
    use std::any::Any;
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
            // let r  = ext_create_timer(true).await;
            // if let Some(v) = r.downcast_ref::<i8>() {
            //     println!("i8 {}", v)
            // } else if let Some(v) = r.downcast_ref::<String>() {
            //     println!("string {}", v)
            // }
        });

    }

    #[test]
    fn test2() {

    }
}
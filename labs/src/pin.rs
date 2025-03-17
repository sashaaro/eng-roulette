use std::any::Any;
use std::future::Future;
pub struct SelfRef {
    s: String,
    s_ref: *const String
}

pub async fn with_pin() {

}

#[cfg(test)]
mod tests {
    use std::mem;
    use std::ops::Deref;
    use std::pin::Pin;
    use std::ptr::null;
    use crate::pin::{with_pin, SelfRef};

    fn move_it<T>(a: T) -> T {
        a
    }

    #[tokio::test]
    async fn test_unpin() {
        let mut a = SelfRef {
            s: "hello world".to_string(),
            s_ref: null(),
        };
        a.s_ref = &a.s;

        let s_ref = a.s_ref;

        unsafe {
            assert_eq!("hello world", (*s_ref).clone());
        }
        _ = move_it(a);

        unsafe {
            dbg!((*s_ref).clone());
            assert_ne!("hello world", (*s_ref).clone());
        }
    }
    #[tokio::test]
    async fn test_pin() {
        let mut a = SelfRef {
            s: "hello world".to_string(),
            s_ref: null(),
        };
        let mut a = Box::pin(a); // Оборачиваем в `Pin<Box<T>>`
        a.s_ref = &a.s; // Теперь можно безопасно ссылаться

        let s_ref = a.s_ref;

        unsafe {
            assert_eq!("hello world", (*s_ref).clone());
        }

        let moved = a.s;
        unsafe {
            dbg!((*s_ref).clone());
        }
    }
}
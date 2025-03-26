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

        //let moved = a.s;
        // unsafe {
        //     dbg!((*s_ref).clone());
        // }
    }


    #[derive(PartialEq, Debug, Default)]
    struct Unpinned { // Unpin это маркерный трей (marker trait), который указывает, можно ли перемещать (move) тип в памяти без особых ограничений.
        s: String,
    }

    #[tokio::test]
    async fn test_deref_default_unpinned() {
        move_it::<&dyn Unpin>(&Unpinned::default()); // is unpin by default

        let u = &Unpinned { s: "a".to_string() };
        let mut p = Pin::new(u);
        let p2: &Unpinned = p.deref();
        // let mut m = p.deref_mut();

        assert_eq!(*u, *p2);
    }

    #[derive(Default)]
    struct NotUnpinned { // self ref
        //cell: RefCell<i8>
        _pin: PhantomPinned
    }
    // impl !Unpin for NotUnpinned {}


    #[tokio::test]
    async fn test_pin2() {
        // move_it::<&dyn Unpin>(&NotUnpinned::default()); // within `NotUnpinned`, the trait `Unpin` is not implemented for `PhantomPinned`

        let pinned = NotUnpinned::default();
        move_it(pinned);

        let pinned = NotUnpinned::default();
        let mut p = Box::pin(pinned);
        let mut m = p.deref();
        // let mut m = p.deref_mut();
        // let p = Box::new(pinned);
        // let p2: &Unpinned = p.deref();
        // let u = &Unpinned{s: "b".to_string()};
    }

    #[derive(Clone, PartialEq, Debug)]
    struct SelfRef2 {
        data: String,
        reference: *const String, // Указатель на data
    }

    #[test]
    fn test_pin22() {
        let mut s = SelfRef2 {
            data: String::from("Hello"),
            reference: std::ptr::null(),
        };

        s.reference = &s.data; // Сохраняем ссылку на `data`

        let reff = s.reference;
        move_it(s); // 🔴 Здесь структура перемещается (move)

        // `s.reference` теперь указывает на уже несуществующую память!
        let unsafe_value = unsafe { (*reff).clone() };
        assert_ne!("Hello", unsafe_value);
        // dbg!(unsafe_value);
    }


    #[derive(Clone, PartialEq, Debug)]
    struct PinnedSelfRef2 {
        data: String,
        reference: *const String, // Указатель на data
        _pin: PhantomPinned
    }

    //#[tokio::test]
    async fn test_pin223() {
        let mut s = PinnedSelfRef2 {
            data: String::from("Hello"),
            reference: std::ptr::null(),
            _pin: PhantomPinned,
        };
        s.reference = &s.data; // Сохраняем ссылку на `data`

        let reff = s.reference;
        let s = Box::pin(s);
        move_it(s);

        // `s.reference` теперь указывает на уже несуществующую память!
        assert_ne!("Hello", unsafe { (*reff).clone() });
        dbg!(unsafe { (*reff).clone() });


        // let mut pinned = Box::pin(s);
        // pinned.reference = &pinned.data;
        //
        //
        // let reff = pinned.reference;
        // move_it(pinned);
        //
        // assert_eq!("Hello", unsafe{(*reff).clone()});
        // dbg!(unsafe{(*reff).clone()});
    }


    #[test]
    fn test_addr_tracker() {
        #[derive(Default)]
        struct AddrTracker(Option<usize>);

        impl AddrTracker {
            // If we haven't checked the addr of self yet, store the current
            // address. If we have, confirm that the current address is the same
            // as it was last time, or else panic.
            fn check_for_move(&mut self) {
                let current_addr = self as *mut Self as usize;
                match self.0 {
                    None => self.0 = Some(current_addr),
                    Some(prev_addr) => assert_eq!(prev_addr, current_addr),
                }
            }
        }

        // Create a tracker and store the initial address
        let mut tracker = AddrTracker::default();
        tracker.check_for_move();

        let mut tracker = tracker;

        //tracker.check_for_move();
    }

    fn test_fixed_addr_tracker() {
        #[derive(Default)]
        struct AddrTracker {
            prev_addr: Option<usize>,
            // remove auto-implemented `Unpin` bound to mark this type as having some
            // address-sensitive state. This is essential for our expected pinning
            // guarantees to work, and is discussed more below.
            _pin: PhantomPinned,
        }
        impl AddrTracker {
            fn check_for_move(self: Pin<&mut Self>) {
                let current_addr = &*self as *const Self as usize;
                match self.prev_addr {
                    None => {
                        // SAFETY: we do not move out of self
                        let self_data_mut = unsafe { self.get_unchecked_mut() };
                        self_data_mut.prev_addr = Some(current_addr);
                    },
                    Some(prev_addr) => assert_eq!(prev_addr, current_addr),
                }
            }
        }

        let tracker = AddrTracker::default();
        let tracker = move_it(tracker);
        let mut ptr_to_pinned_tracker: Pin<&mut AddrTracker> = pin!(tracker);
        ptr_to_pinned_tracker.as_mut().check_for_move();
        ptr_to_pinned_tracker.as_mut().check_for_move();

    }
}
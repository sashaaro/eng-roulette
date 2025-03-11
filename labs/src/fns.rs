

#[derive(Clone)]
struct Container {
    name: String,
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::fmt;
    use std::rc::Rc;
    use crate::fns::Container;

    #[test]
    fn test_fn() {
        let simple_callback: fn() -> &'static str = || { // Fn
            return "simple callback"
        };

        let run_callback = |callback: &dyn Fn() -> &'static str| -> &str {
            let s = callback();
            s
        };
        assert_eq!(run_callback(&simple_callback), "simple callback");

        // ------------------

        let mut container = Container {
            name: "test".to_string(),
        };
        let mut mut_callback = || { // FnMut
            container.name.push_str(" muted")
        };

        let run_callback = |callback: &mut dyn FnMut()| {
            callback();
        };

        run_callback(&mut mut_callback);

        assert_eq!(container.name, "test muted");

        // ------------------

        let container = Rc::new(RefCell::new(container.clone()));

        let mut c =  Rc::clone(&container);
        let once_callback = move || { // FnOnce
           c.borrow_mut().name.push_str(" once");
        };

        let run_callback = |callback: Box<dyn FnOnce()>| {
            callback();
        };

        run_callback(Box::new(once_callback));

        assert_eq!(container.borrow().name, "test muted once");
    }
}
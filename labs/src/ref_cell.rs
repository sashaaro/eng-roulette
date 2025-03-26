
#[cfg(test)]
mod tests {
    use std::any::Any;
    use std::cell::{Ref, RefCell};
    use std::marker::PhantomPinned;
    use std::mem;
    use std::ops::{Deref, DerefMut};
    use std::pin::{pin, Pin};
    use std::ptr::{eq, null};
    use std::rc::Rc;
    use crate::pin::{SelfRef};

    #[derive(Debug)]
    struct Person {
        name: String,
        age: u32,
    }

    #[tokio::test]
    async fn test_ref_cell() {

        // Rc — для нескольких владельцев
        // RefCell — для внутренней мутабельности
        let person = Rc::new(RefCell::new(Person {
            name: "Alex".to_string(),
            age: 30,
        }));

        // Клонируем Rc, чтобы у нас было два владельца
        let p1 = Rc::clone(&person);
        let p2 = Rc::clone(&person);

        // Меняем данные через p1
        {
            let mut p1_borrow = p1.borrow_mut();
            p1_borrow.age += 1;
        }

        // Читаем данные через p2
        {
            let p2_borrow = p2.borrow();
            println!("{} is {} years old", p2_borrow.name, p2_borrow.age);
        }

        // Количество владельцев
        println!("Number of owners: {}", Rc::strong_count(&person));





        let data = RefCell::new(5);

        let _ref1 = data.borrow();      // неизменяемая ссылка
        drop(_ref1);
        let _ref2 = data.borrow_mut();  // попытка получить мутабельную ссылку

        // println!("{:?} {:?}", _ref1, _ref2);
    }


    #[test]
    fn test_ref_cell_vec() {
        let v = RefCell::new(vec![1]);

        let mut inc = 1;
        let mut push2 = || v.borrow_mut().push({inc += 1; inc});

        push2();
        push2();
        push2();

        let mut callback = |call: &mut dyn FnMut()| {
            call();
            call();
        };

        callback(&mut push2);

        let mut callback = || {
            callback(&mut push2);

            return v.clone()
        };
        callback();
        dbg!(v.borrow());
    }

    #[test]

    fn test_ref_cell_vec2() {
        let v = Rc::new(RefCell::new(vec![1]));

        let mut callback = |vv: Rc<RefCell<Vec<i32>>>| {

            let mut inc = 1;

            let v_ref = vv.clone();
            let mut push2 = move || v_ref.borrow_mut().push({inc += 1; inc});

            push2();
            push2();
            push2();

            let mut callback = |call: &mut dyn FnMut()| {
                call();
                call();
            };

            callback(&mut push2);

            let v_ref = vv.clone();

            let mut callback = move || {
                callback(&mut push2);

                // return v_ref.clone().borrow()
                return v_ref.clone()
            };
            callback
        };

        // dbg!(v.deref());

        let mut v = v.clone();
        let mut callback = || -> Ref<Vec<i32>> {
            callback(v.clone());
            v.borrow()
        };
        print_ref(&mut callback);
        dbg!(v);
    }

    fn print_ref<'a>(callback: &mut dyn FnMut() -> Ref<'a, Vec<i32>>) {
        let v = callback();
        dbg!(v);
    }
}
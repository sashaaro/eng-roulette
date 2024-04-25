use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

// pub fn boxx() {
//     let b = Box::new(5);
//
//     //increment_box(&b);
//     //increment(*b);
//     let mut str = "hi".to_string();
//     change_str(&mut str);
//     change_str2(&mut str);
//
//     println!("str: {}", str);
//
//     let b = MyBox{i: 2 as i8};
//     print_i(*b);
// }
//
// pub fn print_i(i: i8) {
//
// }
// struct MyBox<T> {
//     i: T
// }
// impl <T> Deref for MyBox<T> {
//     type Target = T;
//     fn deref(&self) -> &Self::Target {
//         return &self.i
//     }
// }
//
// fn increment(i: i64) {
//     println!("increment {}", i)
// }
//
//
// fn increment_box(i: &mut Box<i64>) {
//     println!("increment {}", i);
// }
//
// fn change_str(i: &mut String) {
//     i.push_str("!!")
//     // *i = "bbbee".to_string()
// }
//
//
// fn change_str2(i: &mut String) {
//     i.push_str("??");
//     // assert_eq!(1,1)
// }
//
// pub fn rccc() {
//     let s = "hi".to_string();
//     let mut s = Rc::new(s);
//     let mut r = Rc::clone(&s);
//     println!("count {}", Rc::strong_count(&s));
//     let v = vec!(Rc::clone(&s),Rc::clone(&s));
//     println!("count {}", Rc::strong_count(&r));
//
//     let r = RefCell::new(MyBox{i: 10});
// }

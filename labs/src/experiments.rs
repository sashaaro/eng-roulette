use std::any::Any;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;
use std::sync::Arc;
use std::thread;
use std::thread::JoinHandle;

struct Service {
    list: Vec<i8>,
}

impl Service {

    fn multiple(self: &mut Self, n: i8) {
        self.list.iter_mut().for_each(|item| {
            *item = *item * n;
        })
    }

    fn safe_print(self: &Arc<Self>) {
        println!("{:?}", self.list)
    }
}

#[test]
fn test_not_standard_type_self_struct_method() {
    let mut s = Service{list: vec![1,2,3]};
    s.multiple(2);

    let ss = Arc::new(s);
    ss.safe_print();
}

trait Drawable {
    fn internal_draw(&self);
    fn get_y(&self) -> &f64;
}

trait UniversalDrawable<'a, T> {
    fn internal_draw(&self);
    fn get_y(&self) -> &'a T;
}

#[derive(Debug)]
struct Circle<'y, T> {
    x: T,
    y: &'y T
}
impl<'y> Circle<'y, f64> {
    fn get_yy(&self) -> &'y f64 {
        self.y
    }
}

impl<'y> Drawable for Circle<'y, f64> {
    fn internal_draw(&self) {
        println!("Drawing a circle {:?}", self.x);
    }

    fn get_y(&self) -> &f64 {
        self.y
    }
}

impl<'y, f64: Debug> UniversalDrawable<'y, f64> for Circle<'y, f64> {
    fn internal_draw(&self) {
        println!("Drawing a circle {:?}", self.x);
    }

    fn get_y(&self) -> &'y f64 {
        self.y
    }
}

fn accept_drawable<'a, 'b>(drawable: &'a dyn Drawable, drawable1: &'b dyn Drawable) -> &'a dyn Drawable {
    drawable.internal_draw();
    drawable1.internal_draw();
    drawable
}

fn print_static<'a>(s: &'a str) {
    println!("{}", s);
}

fn print_static2<'a, T: Display>(s: &'a T) {
    println!("{}", s);
}

fn print_static3<T: Display>(s: &'static T) {
    println!("{}", s);
}

fn print_static4<T: Display + 'static>(s: &T) {
    println!("{}", s);
}
#[test]
fn test_dyn() {
    let y = 0.0;
    let shape = Circle{x:1.0, y: &y };
    let mut shape22 = Circle{x:1.1, y:&y };


    let shape1: &dyn Drawable = &shape;
    let shape2: &dyn Drawable;
    if true {
        let y = 2.0;
        let shape3 = Circle{x:2.0, y: &y };
        let shape3: &dyn Drawable = &shape3;
        shape2 = accept_drawable(shape1, shape3);
        //shape2 = accept_drawable(shape3, shape1);
    } else {
        //let y = 2.0;  // y does not live long enough
        shape22.y = &2.0;
        shape2 = &shape22;
    }

    println!("shape 2 y {}", shape2.get_y());


    let var: Box<dyn Drawable> = Box::new(Circle{x:1.0, y:&y });
    let var: Arc<Box<dyn Drawable>> = Arc::new(Box::new(Circle{x:1.0, y:&y }));
}


type ThreadSpawn<F: FnOnce() -> T + Send + 'static, T: Send + 'static> = fn(f: F) -> JoinHandle<T>;

// fn create_fn() {
//     let a: ThreadSpawn<F, T>;
//     let b = || -> ThreadSpawn<F, T>  {
//         thread::spawn
//     };
//     a = b();
//     a
// }

fn take_any<T: Any + 'static>(value: T) {
    println!("Got a value");
}

trait MyTrait {}

struct Foo<'a> {
    data: &'a str, // содержит ссылку
}
impl<'a> MyTrait for Foo<'a> {}

struct Bar {
    data: String, // содержит ссылку
}
impl MyTrait for Bar {}

fn test_static2() {

    let s = String::from("Hello");

    let bar = Bar {data: s.clone()};
    let boxed: Box<dyn MyTrait> = Box::new(bar);
    take_any(boxed);

    let foo = Foo { data: &s };
    let boxed: Box<dyn MyTrait> = Box::new(foo); // ❌ Ошибка: Foo содержит ссылку с ограниченным lifetime
    //take_any(boxed);
}


#[test]
fn test_weak_pointer() {
    let first_rc = Rc::new(5);
    let first = Rc::downgrade(&first_rc);
    let second = Rc::downgrade(&first_rc);

    assert!(first.ptr_eq(&second));

    let third_rc = Rc::new(5);
    let third = Rc::downgrade(&third_rc);

    assert!(!first.ptr_eq(&third));

    assert_ne!(third.upgrade(), None);

    let four = third.upgrade().unwrap();
    drop(four);


    drop(third_rc);
    assert_eq!(third.upgrade(), None)
}
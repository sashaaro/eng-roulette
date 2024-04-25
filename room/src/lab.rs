use std::fmt::Display;
use std::thread;

trait A {
    fn a(&self) {

    }
}

trait B: A {
    fn b(&self) {
        self.a()
    }
}


#[derive(Clone)]
struct Point<T> {
    x: T,
    y: T,
    meta: String
}


impl<T: Display> Point<T> {
    fn display(&self) {
        println!("Point! {} {}", self.x, self.y);
    }
}

fn ttake_ownership(a: &str) {
    // let handle =  thread::spawn(|| {
    //     thread::sleep(Duration::from_millis(1));
    //     println!("hi number {} from the spawned thread!", a.clone());
    //     thread::sleep(Duration::from_millis(1));
    // });
    // handle.join().unwrap();
}

fn take_ref(a: &str) {

}

fn reff() {
    let mut aa = Point{x: 10, y:12, meta: "1".to_string()};
    take_str_ownership(aa.meta);
    //take_ownership(aa);

    let mut v = vec![1,2,3,4];
    //ttake_ownership(&v[0]);
    //ttake_ownership(v[3]);
    let mut v = vec![
        "1".to_string(),
        "2".to_string(),
        "3".to_string()
    ];
    ttake_ownership(&mut v[0]);
    //ttake_ownership(v[3]);

    let mut p = Point{x: 10, y:12, meta: "1".to_string()};
    print_point(&p);
    print_point(&p);

    (|| {
        p.x = 1;
    })();
    let handle = thread::spawn(move || {
        println!("Here's a vector: {:?}", p.x);
    });
    handle.join().unwrap();

    (|| {
        println!("Hello {}!", p.x);
    })();
    // (|| {
    //     p;
    // })();

    let p_ref = &p;
    print_point(p_ref);
    print_point(p_ref);
    lifetime(p_ref);
    longest(p_ref, &Point{x: 1,y: 2,meta:"1".to_string()});
    longest(p_ref, &Point{x: 1,y: 2,meta:"1".to_string()});

    //let ff = *p_ref;
    let f = (*p_ref).clone();
    take_ownership(f);

    let mut mp = Point{x: 11, y: 16,meta:"1".to_string()};
    mut_lifetime(&mut mp);
    mut_lifetime(&mut mp);

    let mut_mp_ref = &mut mp;
    mut_lifetime(mut_mp_ref);
    mut_lifetime(mut_mp_ref);
    let mp_ref = &mp;
    print_point(mp_ref);

    let args = cli::Args::parse();

    println!("Hello {}!", args.port);
}


fn take_ownership<T: Display >(p: Point<T>) {
    p.display()
}

fn take_str_ownership(p: String) {
    println!("{}", p)
}

fn print_point<T: Display >(p: &Point<T>) {
    p.display()
}

fn lifetime<'a, T: Display>(p: &'a Point<T>) {
    p.display()
}
fn mut_lifetime<'a>(p: &'a mut Point<i32>) {
    p.y += 10
}

fn longest<'a, 'b>(a: &'a Point<i32>, b: &'b Point<i32>) -> &'b Point<i32> {
    b
}


fn main2() {
    // boxx();
    //reff();
    //rccc();

    //return;




    let mut list = vec![1, 2, 3];
    println!("Before defining closure: {:?}", list);

    //let f = &list
    let mut only_borrows = || {
        list.push(4);
        println!("From closure: {:?}", list)
    };

    //println!("Before calling closure: {:?}", list);
    only_borrows();
    println!("After calling closure: {:?}", list);
}
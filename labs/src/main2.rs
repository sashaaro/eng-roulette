use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::thread;

pub fn main2() {

    //arc();
    //rc();

    refCell();
}

fn arc() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]); // определяем общие данные
    // создаем две копии общих данных
    let clone1 = Arc::clone(&data);
    let clone2 = Arc::clone(&data);


    // запускаем 2 потока, которые обращаются к общим данным
    let thread1 = thread::spawn(move || {

        let sum: i32 = clone1.iter().sum();
        println!("Thread 1 - Sum of Data: {}", sum);
    });

    let thread2 = thread::spawn(move || {

        let len: usize = clone2.len();
        println!("Thread 2 - Length of Data: {}", len);
    });

    // ждем завершения потоков
    thread1.join().unwrap();
    thread2.join().unwrap();
}

fn rc() {
    let data = Rc::new(vec![1, 2, 3, 4, 5]); // определяем общие данные
    // создаем две копии общих данных
    let clone1 = Rc::clone(&data);
    let clone2 = Rc::clone(&data);


    // запускаем 2 потока, которые обращаются к общим данным
    let f  = move || {
        let sum: i32 = clone1.iter().sum();
        println!("Thread 1 - Sum of Data: {}", sum);
    };

    f();

    let f = move || {
        let len: usize = clone2.len();
        println!("Thread 2 - Length of Data: {}", len);
    };

    f();
}

fn refCell() {
    let data = RefCell::new(vec![1, 2, 3]); // определяем данные

    let mut data_ref = data.borrow_mut();

    let mut f1 = move || {
        data_ref.push(4);
    };
    let mut data_ref = data.borrow_mut();

    let mut f2 = move || {
        data_ref.push(5);
    };

    f2();
    f1();
    println!("Data: {:?}", data.borrow()); // Data: [1, 2, 3]}

}
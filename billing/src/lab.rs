// use core::fmt;
// use std::error::Error;
// use std::fmt::Formatter;
//
// fn lab() {
//     let a = 5;
//     let b = 10.0;
//     print_number(a);
//     print_number(b);
//     print_number2(a);
//     print_number2(b);
//     // println!("error 1: {:?}", return_error(0));
//     // println!("error 1: {:?}", return_error(1));
//     // println!("error 1: {:?}", return_error(2));
// }
//
//
// fn print_number<T: std::fmt::Display>(x: T) {
//     println!("{}", x);
// }
//
// fn print_number2(x: impl std::fmt::Display) {
//     println!("{}", x);
// }
//
// fn mapp<U, T>(f: impl FnOnce(T)) -> Option<U> {
//     return None
// }
// fn mappp<U, T, F>(f: F) -> Option<U> where F: FnOnce(T) -> U {
//     return None
// }
//
// #[derive(Debug)]
// struct ErrorOne;
// impl Error for ErrorOne{}
// impl fmt::Display for ErrorOne {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "error one")
//     }
// }
//
// #[derive(Debug)]
// struct ErrorTwo;
// impl Error for ErrorTwo{}
// impl fmt::Display for ErrorTwo {
//     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//         write!(f, "error two")
//     }
// }
//
// // fn return_error(input: u8) -> Result<String, impl Error> {
// //     match input {
// //         0 => Err(ErrorOne),
// //         1 => Err(ErrorTwo),
// //         _ => Ok("no error".to_string())
// //     }
// // }?
//
//
//
// // pub fn notify<T: Debug>(item: &T) {
// //     println!("Breaking news! {}", item.summarize());
// // }
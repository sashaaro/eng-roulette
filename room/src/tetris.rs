use core::error::Error;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Write};
use std::fs::Permissions;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use serde::de::Unexpected::Option;
use tracing_subscriber::fmt;

pub async fn create_tetris() {
    let counter = Arc::new(Mutex::new(0));


    let c = counter.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(2000));
            *c.lock().unwrap() += 1;
        }
    });


    let blocks = Arc::new(Mutex::new(Vec::new()));
    let b1 = blocks.clone();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(3000));
            let mut cube = create_cube();
            put_block(&mut cube, Coordinate(8, 1, Arc::new(String::new())));

            b1.lock().unwrap().push(cube);
        }
    });

    let c2 = counter.clone();
    let b2 = blocks.clone();

    loop {
        sleep(Duration::from_millis(500));

        let mut coordinates: Vec<Coordinate> = Vec::new();
        coordinates.append(&mut draw_line((1, 1), (1, 16)).unwrap());
        coordinates.append(&mut draw_line((2, 1), (16, 1)).unwrap());

        coordinates.append(&mut draw_line((16, 1), (16, 16)).unwrap());
        coordinates.append(&mut draw_line((2, 16), (16, 16)).unwrap());


        coordinates.push(Coordinate(4,4, Arc::new(c2.lock().unwrap().to_string() + " ")));

        for block in b2.lock().unwrap().iter().clone() {
            for x in block.iter() {
                coordinates.push(x.clone());
            }
        }

        let render = Render::new();
        render.render(&coordinates);

        // println!("|");
        // println!("| {}!", c2.lock().unwrap());
        // println!("|");
        // println!("_____________");
    }

}

fn create_cube() -> Vec<Coordinate> {
    vec![Coordinate(1, 1, Arc::new("++")), Coordinate(1, 2, Arc::new("++")), Coordinate(1, 3, Arc::new("++")), Coordinate(1, 4, Arc::new("++"))]
}

fn spawn_block() {

}

fn put_block(block: &mut Vec<Coordinate>, start :Coordinate) {
    for p in block {
        p.0 += start.0 - 1;
        p.1 += start.1 - 1;
    }
}

struct MyError<'a> {
    msg : &'a str
}

impl Debug for MyError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg)
    }
}

impl std::fmt::Display for MyError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg)
    }
}

impl Error for MyError<'_> {

}



fn draw_line(start: (i32, i32), end: (i32, i32)) -> Result<(Vec<Coordinate>), Box<dyn Error>> {
    if start.0 != end.0 && start.1 != end.1 {
        return Err(Box::new(MyError{msg : "Only horizontal and vertical lines are supported"}));
    }

    if start.0 == end.0 {
        draw_vertical_line(start.0, start.1, end.1)
    } else {
        draw_horizontal_line(start.1, start.0, end.0)
    }
}

fn draw_vertical_line(start_x: i32, start_y: i32, end_y: i32) -> Result<Vec<Coordinate>, Box<dyn Error>> {
    let mut res = Vec::new();
    if start_y < end_y {
        for n in start_y..=end_y {
            res.push(Coordinate(start_x, n, Arc::new(WALL)))
        }
    } else {
        for n in end_y..=start_x {
            res.push(Coordinate(start_x, n, Arc::new(WALL)))
        }
    }

    Ok(res)
}

fn draw_horizontal_line(start_x: i32, start_y: i32, end_y: i32) -> Result<Vec<Coordinate>, Box<dyn Error>> {
    let mut res = Vec::new();
    if start_y < end_y {
        for n in start_y..=end_y {
            res.push(Coordinate(n, start_x, Arc::new(WALL)))
        }
    } else {
        for n in end_y..=start_x {
            res.push(Coordinate(n, start_x, Arc::new(WALL)))
        }
    }

    Ok(res)
}
// y | _ _ _ _
// 1 | 1 2 3 4 x
// 2 |
// 3 |

struct Render {

}


#[derive(Clone)]
struct Coordinate (i32, i32, Arc<dyn Display + Send + Sync>);

const WALL: &str = "++";

impl Render {

    pub fn new() -> Self {
        Self {}
    }

    pub fn render(&self, coordinates: &Vec<Coordinate>) {
        self.clear();
        let mut coordinatesMap : HashMap<i32, Vec<&Coordinate>> = HashMap::new();

        for c in coordinates {
            let mut v = coordinatesMap.get_mut(&c.1);
            if v.is_some() {
                v.unwrap().push(c);
            } else {
                let mut vv: Vec<&Coordinate> = Vec::new();
                vv.push(c);
                coordinatesMap.insert(c.1, vv);
            }
        }

        for y in 1..=16 {
            for x in 1..=16 {
                if coordinatesMap.get(&y).is_some() {

                    let mut res = None;
                    for c in coordinatesMap.get(&y).unwrap().iter() {
                        if c.0 == x {
                            res = Some(c);
                            break;
                        }
                    }

                    if res.is_some() {
                        print!("{}", res.unwrap().2);
                    } else {
                        print!("  ");
                    }
                } else {
                    print!("  ");
                }
            }
            println!("");
        }
    }

    fn clear(&self) {
        print!("\x1B[2J\x1B[1;1H");
    }
}

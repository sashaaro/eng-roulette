use core::error::Error;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Write};
use std::fs::Permissions;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use tracing_subscriber::fmt;
use std::option::Option;
use tokio::io;
use tokio::io::{AsyncReadExt, Stdin};
use crate::tetris::Movement::Down;

pub async fn create_tetris() {
    let counter = Arc::new(Mutex::new(0));


    let c = counter.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(2000));
            *c.lock().unwrap() += 1;
        }
    });


    let blocks: Arc<Mutex<Vec<Element>>> = Arc::new(Mutex::new(Vec::new()));
    let mut current_block: Arc<Mutex<Option<Element>>> = Arc::new(Mutex::new(None));

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(1000));

            //current_block
        }
    });

    let cur_block = Arc::clone(&current_block);
    tokio::spawn(async move {
        sleep(Duration::from_millis(1000));
        let mut cb = cur_block.lock().unwrap();

        let mut new_element = create_cube();

        put_block(&mut new_element, Coordinate(8, 2, Arc::new(String::new())));
        *cb = Some(new_element);
    });

    let cur_block = Arc::clone(&current_block);
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(1000));


            let mut c = cur_block.lock().unwrap();
            if c.is_some() {
                let cc = &mut c.as_mut().unwrap();
                cc.next_movement = Some(Down);

                // fail_down(cc);
            }
        }
    });

    let c2 = counter.clone();
    let blocks2 = Arc::clone(&blocks.clone());

    let cur_block = Arc::clone(&current_block);


    let walls: Vec<Element> = vec!(
        draw_line((1, 1), (1, 16)).unwrap(),
        draw_line((2, 1), (16, 1)).unwrap(),
        draw_line((16, 1), (16, 16)).unwrap(),
        draw_line((2, 16), (16, 16)).unwrap()
    );

    tokio::spawn(async move {
        let mut stdin = io::stdin();
        loop {
            let mut buffer = [0;1];
            stdin.read_exact(&mut buffer).await.unwrap();

            if buffer[0] == 27 {
                break;
            }
            println!("You have hit: {:?}", buffer[0]);

        }
    });

    let build_coordinates = || -> Vec<Coordinate> {
        let mut coordinates: Vec<Coordinate> = Vec::new();
        walls.clone().into_iter().for_each(|x| {
            coordinates.extend_from_slice(x.block.as_slice());
        });

        coordinates.push(Coordinate(3,3, Arc::new(c2.lock().unwrap().to_string() + " ")));
        let mut c = cur_block.lock().unwrap();

        if c.is_some() {
            for x in c.as_ref().unwrap().block.iter() {
                coordinates.push(x.clone());
            }
        }

        for block in blocks2.lock().unwrap().iter_mut() {
            if block.next_movement.is_some() {
                match block.next_movement {
                    Some(Down) => {
                        fail_down(block);
                    },
                    _ => {}
                }
                block.next_movement = None;
            }
            for x in block.block.iter() {
                coordinates.push(x.clone());
            }
        }

        coordinates
    };


    let render = Render::new();
    loop {
        sleep(Duration::from_millis(500));
        render.render(&build_coordinates());
    }
}

fn fail_down(block: &mut Element) {
    for x in block.block.iter_mut() {
        x.1 += 1;
    }
}

fn create_cube() -> Element {
    Element::new(vec![Coordinate(1, 1, Arc::new("++")), Coordinate(1, 2, Arc::new("++")), Coordinate(2, 1, Arc::new("++")), Coordinate(2, 2, Arc::new("++"))])
}

fn spawn_block() {

}

fn put_block(block: &mut Element, start :Coordinate) {
    for p in block.block.iter_mut() {
        p.0 += start.0 - 1;
        p.1 += start.1 - 1;
    }
}

#[derive(Clone)]
enum Movement {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}

#[derive(Clone)]
struct Element {
    block: Vec<Coordinate>,
    next_movement: Option<Movement>,
}

impl Element {
    fn new(block: Vec<Coordinate>) -> Element {
        Element {
            block,
            next_movement: None,
        }
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



fn draw_line(start: (i32, i32), end: (i32, i32)) -> Result<(Element), Box<dyn Error>> {
    if start.0 != end.0 && start.1 != end.1 {
        return Err(Box::new(MyError{msg : "Only horizontal and vertical lines are supported"}));
    }

    let r;
    if start.0 == end.0 {
        r = draw_vertical_line(start.0, start.1, end.1)
    } else {
        r = draw_horizontal_line(start.1, start.0, end.0)
    }
    r.map(|r| -> Element { Element::new(r) })
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
struct Coordinate (i32, i32, Arc<dyn Display + Send + Sync>); // x, y

const WALL: &str = "++";


fn coordinates_to_hashmap(coordinates: &Vec<Coordinate>) -> HashMap<i32, Coordinate> {
    let mut hashmap = HashMap::new();
    for c in coordinates {
        let mut v = hashmap.get_mut(&c.1);
        if v.is_some() {
            v.unwrap().push(c);
        } else {
            let mut vv: Vec<&Coordinate> = Vec::new();
            vv.push(c);
            hashmap.insert(c.1, vv);
        }
    }

    hashmap
}

impl Render {

    pub fn new() -> Self {
        Self {}
    }

    pub fn render_coordinates(&self, coordinates: &Vec<Coordinate>) {
        let mut coordinates_map = coordinates_to_hashmap(coordinates);

        for y in 1..=16 {
            for x in 1..=16 {
                if coordinates_map.get(&y).is_some() {

                    let mut res = None;
                    for c in coordinates_map.get(&y).unwrap().iter() {
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
    pub fn render(&self, vec1: &Vec<Coordinate>) {
        self.render_coordinates(vec1);
        self.clear();
    }

    fn clear(&self) {
        print!("\x1B[2J\x1B[1;1H");
    }
}

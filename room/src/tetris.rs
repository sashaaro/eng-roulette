use core::error::Error;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Write};
use std::fs::Permissions;
use std::ops::Deref;
use std::sync::{Arc, Mutex};
use tokio::time::sleep;
use std::time::Duration;
use tracing_subscriber::fmt;
use std::option::Option;
use tokio::io;
use tokio::io::{AsyncReadExt, Stdin};
use tokio::sync::mpsc::{channel, Sender};
use crate::tetris::Movement::Down;

pub async fn create_tetris() {
    let counter = Arc::new(Mutex::new(0));


    let c = counter.clone();
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(2000)).await;
            *c.lock().unwrap() += 1;
        }
    });


    let blocks: Arc<Mutex<Vec<Element>>> = Arc::new(Mutex::new(Vec::new()));
    let mut current_block: Arc<Mutex<Option<Element>>> = Arc::new(Mutex::new(None));

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(1000)).await;

            //current_block
        }
    });

    let (next_block_sender, mut next_block_receiver) = channel(3);

    let next_block_sx =  next_block_sender.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(1000)).await;
        let new_element = create_cube();
        next_block_sx.send(new_element).await.expect("TODO: panic message");
    });

    let mut cur_block = Arc::clone(&current_block);
    tokio::spawn(async move {
        loop {
            let mut new_element = next_block_receiver.recv().await;
            if new_element.is_none() {
                break;
            }
            let mut new_element = new_element.unwrap();

            put_block(&mut new_element, Coordinate { x: 8, y: 2, b: Arc::new(String::new()) });
            *cur_block.lock().unwrap() = Some(new_element);
        }
    });

    let cur_block = Arc::clone(&current_block);
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_millis(1000)).await;


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

    let next_block_sx =  next_block_sender.clone();
    let cur_block = Arc::clone(&current_block);



    let mut render = Render::new(next_block_sx, cur_block, walls, c2, blocks2);
    loop {
        sleep(Duration::from_millis(500)).await;
        render.render().await;
    }
}

fn fail_down(block: Element) -> Vec<Coordinate> {
    let mut res = Vec::new();
    for x in block.block.iter() {
        res.push(Coordinate{x: x.x, y: x.y + 1, b: Arc::new(WALL)});
    }
    res
}

fn create_cube() -> Element {
    Element::new(vec![
        Coordinate{x: 1, y: 1, b: Arc::new("++")},
        Coordinate{x: 1, y: 2, b: Arc::new("++")},
        Coordinate{x: 2, y: 1, b: Arc::new("++")},
        Coordinate{x: 2, y: 2, b: Arc::new("++")},
    ])
}

fn spawn_block() {

}

fn put_block(block: &mut Element, start :Coordinate) {
    for p in block.block.iter_mut() {
        p.x += start.x - 1;
        p.y += start.y - 1;
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
            res.push(Coordinate{x: start_x, y: n, b: Arc::new(WALL)})
        }
    } else {
        for n in end_y..=start_x {
            res.push(Coordinate{x: start_x, y: n, b: Arc::new(WALL)})
        }
    }

    Ok(res)
}

fn draw_horizontal_line(start_x: i32, start_y: i32, end_y: i32) -> Result<Vec<Coordinate>, Box<dyn Error>> {
    let mut res = Vec::new();
    if start_y < end_y {
        for n in start_y..=end_y {
            res.push(Coordinate{x: n, y: start_x, b: Arc::new(WALL)})
        }
    } else {
        for n in end_y..=start_x {
            res.push(Coordinate{x: n, y: start_x, b: Arc::new(WALL)})
        }
    }

    Ok(res)
}
// y | _ _ _ _
// 1 | 1 2 3 4 x
// 2 |
// 3 |

struct Render {
    cur_block: Arc<Mutex<Option<Element>>>,
    next_block_sx: Sender<Element>,
    walls: Vec<Element>,
    c2: Arc<Mutex<i32>>,
    blocks2: Arc<Mutex<Vec<Element>>>,

    elements: Vec<Element>,
}


#[derive(Clone)]
struct Coordinate {
    x: i32,
    y: i32,
    b: Arc<dyn Display + Send + Sync>
}

const WALL: &str = "++";


fn coordinates_to_hashmap(coordinates: &Vec<Coordinate>) -> HashMap<i32, Vec<&Coordinate>> { // y => x
    let mut hashmap : HashMap<i32, Vec<&Coordinate>> = HashMap::new();
    for c in coordinates {
        let mut v = hashmap.get_mut(&c.y);
        if v.is_some() {
            v.unwrap().push(c);
        } else {
            let mut vv: Vec<&Coordinate> = Vec::new();
            vv.push(c);
            hashmap.insert(c.y, vv);
        }
    }

    hashmap
}

impl Render {

    pub fn new(next_block_sx: Sender<Element>, cur_block: Arc<Mutex<Option<Element>>>, walls: Vec<Element>, c2: Arc<Mutex<i32>>, blocks2: Arc<Mutex<Vec<Element>>>) -> Self {
        Self {
            next_block_sx,
            cur_block,
            walls,
            c2,
            blocks2,
            elements: vec!(),
        }
    }



    pub async fn build_coordinates(&mut self) -> Vec<Coordinate> {
            let mut coordinates: Vec<Coordinate> = Vec::new();
            self.walls.clone().into_iter().for_each(|x| {
                coordinates.extend_from_slice(x.block.as_slice());
            });

            coordinates.push(Coordinate{x: 3, y: 3, b: Arc::new(self.c2.lock().unwrap().to_string() + " ")});

            for block in self.elements.iter() {
                for x in block.block.iter() {
                    coordinates.push(x.clone());
                }
            }

            let mut c = self.cur_block.lock().unwrap();

            let core_map = coordinates_to_hashmap(&coordinates);

            let mut lets_move = true;
            if c.is_some() {
                let mut block = c.as_mut().unwrap();
                match block.next_movement {
                    Some(Down) => {
                        let moved_block = fail_down(block.clone());
                        for c in moved_block.iter() {
                            let v = core_map.get(&c.y);
                            if v.is_some() {
                                if v.unwrap().iter().map(|x| x.x).filter(|x| *x == c.x).count() > 0 {
                                    lets_move = false;
                                    break
                                }
                            }
                        }
                        if lets_move {
                            block.block = moved_block;
                        }
                    },
                    _ => {

                    }
                }
                block.next_movement = None;
                for x in c.as_ref().unwrap().block.iter() {
                    coordinates.push(x.clone());
                }
            }
            if !lets_move {
                // tokio::spawn(async {
                //     sleep(Duration::from_millis(1000)).await;

                self.elements.push(c.as_ref().unwrap().clone());
                *c = None;

                let new_element = create_cube();
                self.next_block_sx.send(new_element).await.unwrap();
                // });

            }

            for block in self.blocks2.lock().unwrap().iter() {
                for x in block.block.iter() {
                    coordinates.push(x.clone());
                }
            }

            coordinates
    }



    pub async fn render_coordinates(&mut self) {
        let coordinates = self.build_coordinates().await;
        let mut coordinates_map = coordinates_to_hashmap(&coordinates);

        for y in 1..=16 {
            for x in 1..=16 {
                if coordinates_map.get(&y).is_some() {

                    let mut res = None;
                    for c in coordinates_map.get(&y).unwrap().iter() {
                        if c.x == x {
                            res = Some(c);
                            break;
                        }
                    }

                    if res.is_some() {
                        print!("{}", res.unwrap().b);
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
    pub async fn render(&mut self) {
        self.render_coordinates().await;
        self.clear();
    }

    fn clear(&self) {
        print!("\x1B[2J\x1B[1;1H");
    }
}

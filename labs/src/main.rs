use std::fmt::Display;

fn main() {
    let buffer = String::from("hello");
    foo(&buffer);
}

fn foo<'a>(buffer: &'a String) {
    let parser = StreamParser::new(buffer);
    let buffer = parser.get_buffer();
    let offset = parser.get_offset();

    // use_string(buffer);
    // take_parser(&parser);

    use_string(&parser);
    use_string(buffer);

    use_string(parser);
    use_string(buffer);
    // let parser_ref = &parser;
    // take_parser(parser_ref);
    // take_parser(parser_ref);


    //use_offset(offset);

    let a = "hello".to_string();
    let b = "bob".to_string();
    let max = StreamParser::max(&a, &b);
    println!("{}", max);

    let a = StreamParser::new(&a);
    let b = StreamParser::new(&b);
    let max = StreamParser::max_buffer(&a, &b);
    println!("{}", max.buffer);
}

fn use_offset(p0: &usize) {
    todo!()
}

fn use_string<T: Display>(p0: T) {
    println!("use string {}", p0);
}

fn take_parser(stream_parser: &StreamParser<'_>) {
    println!("take parser {:?}", stream_parser);
}

#[derive(Debug)]
struct StreamParser<'a> {
    buffer: &'a String,
    offset: usize,
}

impl Display for StreamParser<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StreamParser {{ buffer: {}, offset: {} }}", self.buffer, self.offset)
    }
}

impl<'a> StreamParser<'a> {
    fn new(buffer: &'a String) -> Self {
        Self {
            buffer,
            offset: 0
        }
    }

    fn max<'g>(a: &'g String, b: &'g String) -> &'g String {
        if a.len() > b.len() {
            a
        } else {
            b
        }
    }

    fn max_buffer<'g>(a: &'g Self, b: &'g Self) -> &'g Self {
        if a.buffer.len() > b.buffer.len() {
            a
        } else {
            b
        }
    }
}

impl<'a, 'b> StreamParser<'a> {
    fn parser(&'a mut self, step: usize) {
        self.offset += step
    }

    fn get_buffer(&self) -> &String {
        self.buffer
    }

    fn get_offset(&self) -> &usize {
        &self.offset
    }
}
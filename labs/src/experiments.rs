use std::sync::Arc;

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
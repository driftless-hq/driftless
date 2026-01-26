use minijinja::{Value, Error};

fn main() {
    let error = Error::new(minijinja::ErrorKind::InvalidOperation, "test error");
    let value: Value = error.into();
    println!("{:?}", value);
}

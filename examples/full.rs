use std::path::Path;

use chainschema_gen::Schema;

fn main() {
    load_schema_file();
}

fn load_schema_file() {
    let path = Path::new("examples/schema.chain");
    let schema = Schema::load(path).unwrap();
    println!("{:#?}", schema);
}

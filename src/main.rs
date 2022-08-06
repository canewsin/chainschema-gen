mod schema;
mod types;

use std::path::Path;

use schema::Schema;

fn main() {
    load_schema_file();
}

fn load_schema_file() {
    let path = Path::new("schema/schema.chain");
    let schema = Schema::load(path).unwrap();
    println!("{:#?}", schema);
}

use serde::Serialize;

pub fn main() {
    let cat = Cat {
        name: "Nyan".to_owned(),
        age: 6,
        breed: Breed::Persian,
    };

    let serialized_cat = serde_json::to_string_pretty(&cat).unwrap();

    println!("Serialized cat = {serialized_cat}")
}

#[derive(Serialize)]
struct Cat {
    name: String,
    age: u8,
    breed: Breed,
}

#[derive(Serialize)]
enum Breed {
    Persian,
    #[allow(dead_code)]
    Siamese,
}

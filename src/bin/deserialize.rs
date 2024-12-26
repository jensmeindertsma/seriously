use serde::Deserialize;

pub fn main() {
    let string = "
        {
        \"name\": \"Rocko\",
        \"age\": 4,
        \"breed\": \"Husky\"
        }
    ";

    let deserialized_dog: Dog = serde_json::from_str(string).unwrap();

    println!("Deserialized dog age = {}", deserialized_dog.age)
}

#[derive(Deserialize)]
struct Dog {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    age: u8,
    #[allow(dead_code)]
    breed: Breed,
}

#[derive(Deserialize)]
enum Breed {
    Husky,
    #[allow(dead_code)]
    Teckel,
}

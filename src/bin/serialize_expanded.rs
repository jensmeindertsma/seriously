use serde::{ser::SerializeStruct, Serialize, Serializer};

pub fn main() {
    let cat = Cat {
        name: "Nyan".to_owned(),
        age: 6,
        breed: Breed::Persian,
    };

    let serialized_cat = serde_json::to_string_pretty(&cat).unwrap();

    println!("Serialized cat = {serialized_cat}");
}

struct Cat {
    name: String,
    age: u8,
    breed: Breed,
}

impl Serialize for Cat {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut serde_state =
            Serializer::serialize_struct(serializer, "Cat", false as usize + 1 + 1 + 1)?; // very strange way to say `3 fields`!

        SerializeStruct::serialize_field(&mut serde_state, "name", &self.name)?;
        SerializeStruct::serialize_field(&mut serde_state, "age", &self.age)?;
        SerializeStruct::serialize_field(&mut serde_state, "breed", &self.breed)?;

        SerializeStruct::end(serde_state)
    }
}

enum Breed {
    Persian,
    #[allow(dead_code)]
    Siamese,
}

impl Serialize for Breed {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match *self {
            Breed::Persian => {
                Serializer::serialize_unit_variant(serializer, "Breed", 0u32, "Persian")
            }
            Breed::Siamese => {
                Serializer::serialize_unit_variant(serializer, "Breed", 1u32, "Siamese")
            }
        }
    }
}

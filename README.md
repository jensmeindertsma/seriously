# seriously

How does the serde crate actually work? And I mean particularly those magic `#[derive(Deserialize, Serialize)]` macros! Stick around to find out!

## cargo-expand

I've used `cargo-expand to unpack all the derive macros in use, and prettified what code was generated. Take a look for yourself!

- [deserialize.rs](./src/bin/deserialize.rs)
- [deserialize_expanded.rs](./src/bin/deserialize_expanded.rs)

- [serialize.rs](./src/bin/serialize.rs)
- [serialize_expanded.rs](./src/bin/serialize_expanded.rs)

## Serialization explained

Let's start with serialization, as this is the simpler side of things.

```rs
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
```

This code is then expanded to:

```rs
use serde::{ser::SerializeStruct, Serialize, Serializer};

pub fn main() {
    let cat = Cat {
        name: "Nyan".to_owned(),
        age: 6,
        breed: Breed::Persian,
    };

    // 1. `to_string_pretty` is generic over anything that implements `Serialize`.
    //
    // 2. It will pass a JSON serializer (struct that implements `Serializer`) to `Cat::serialize`.
    //
    // 3. `Cat::serialize` (implemented below) will then call methods on that JSON serializer according
    // to its own type and also for any types it holds.
    //
    // 4. This means the JSON serializer struct, for each request to serialize a type or field,
    // will append new bytes to the string it returns at the end.
    let serialized_cat = serde_json::to_string_pretty(&cat).unwrap();

    println!("Serialized cat = {serialized_cat}");
}

struct Cat {
    name: String,
    age: u8,
    breed: Breed,
}

impl Serialize for Cat {
    // 5. Here in the implementation of `Serialize` for Cat we can see it is generic over the `Serializer`.
    //     What does that trait look like?
    //
    // pub trait Serializer: Sized {
    //      fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error>;
    //      fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error>;
    // }
    //
    // It defines a method for each and every single type in the "Serde data model". What's that?
    // It is a simplified form of Rust's type system that allows you to construct any type.
    // Read more: https://serde.rs/data-model.html

    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // 6. The first step is to signal to the serializer that we want to "open" a struct.
        let mut serde_state =
            Serializer::serialize_struct(serializer, "Cat", false as usize + 1 + 1 + 1)?; // very strange way to say `3 fields`!

        // 7. Now, we serialize each field in order. The syntax here might be a little weird,
        // but its identical to `serde_state.serialize_field("name", &self.name)`. Instead a
        // `fully-qualified` syntax is used to prevent trait method name conflicts.
        SerializeStruct::serialize_field(&mut serde_state, "name", &self.name)?;
        SerializeStruct::serialize_field(&mut serde_state, "age", &self.age)?;

        // 8. The `self.breed` field is not of a primitive type like the previous fields,
        // but because `serialize_field` takes anything that implements `Serialize`, which
        // is implemented for u8 just as well as for `Breed` it just works. If `Breed` was another
        // struct we'd enter a new depth level / recursion.
        SerializeStruct::serialize_field(&mut serde_state, "breed", &self.breed)?;

        // 9. We lastly signal the end of the struct.
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
        // 10. Finally, for the implementation of `Breed::serialize`, each variant is serialized
        // as a number identifying the variant, and the name of the variant is stored as well.
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
```

## Deserialization

This one is a lot more intricate. Nontheless, let's start with the original source code, slightly modified:

```rs
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
```

**TODO**: deserialization expansion explained

## Resources

I learnt most of what's here by watching this really good deep-dive by `@jonhoo`:

<iframe width="560" height="315" src="https://www.youtube.com/embed/BI_bHCGRgMY?si=Gf5gpiOeeYcFFACy" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

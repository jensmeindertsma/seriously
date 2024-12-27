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

I've expanded the code and added comments to understand how the serialization works. This one is fairly easy to grasp as it's quite linear, just encode each field in the struct with a key and then a value (recursively):

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

This one is a lot more intricate. Nonetheless, let's start with the original source code, slightly modified:

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

    // `from_str` invokes deserialize on the target type which is `Dog`
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

This expands to the following (prettified and with notes):

```rs
use core::fmt::{self, Formatter};
use std::marker::PhantomData;

use serde::{
    de::{EnumAccess, Error, IgnoredAny, MapAccess, SeqAccess, Unexpected, VariantAccess, Visitor},
    Deserialize, Deserializer,
};

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

struct Dog {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    age: u8,
    #[allow(dead_code)]
    breed: Breed,
}

impl<'de> Deserialize<'de> for Dog {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Zero,
            One,
            Two,
            Ignore,
        }

        struct FieldVisitor;

        // 4. The `FieldVisitor` here implements 3 ways to deserialize "fields":
        // - as a number
        // - as a string
        // - as a sequence of bytes.
        // Check the Visitor trait for all of the methods: https://github.com/serde-rs/serde/blob/cb6eaea151b831db36457fff17f16a195702dad4/serde/src/de/mod.rs#L1284
        // Not all methods are required, they all have default implementations that either do basic
        // deserialization or just return an error.
        impl Visitor<'_> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                Formatter::write_str(formatter, "field identifier")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    0u64 => Ok(Field::Zero),
                    1u64 => Ok(Field::One),
                    2u64 => Ok(Field::Two),
                    _ => Ok(Field::Ignore),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    "name" => Ok(Field::Zero),
                    "age" => Ok(Field::One),
                    "breed" => Ok(Field::Two),
                    _ => Ok(Field::Ignore),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    b"name" => Ok(Field::Zero),
                    b"age" => Ok(Field::One),
                    b"breed" => Ok(Field::Two),
                    _ => Ok(Field::Ignore),
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }

        struct DogVisitor<'de> {
            marker: PhantomData<Dog>,
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de> Visitor<'de> for DogVisitor<'de> {
            type Value = Dog;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                Formatter::write_str(formatter, "struct Dog")
            }

            // 2. This method know about the order of fields and so it requests the values from the
            // deserializer by which it was invoked through the `SeqAccess` interface. It builds the
            // Dog struct from these fields and gives it back to the deserializer (or an error).
            // It is really important to notice that `next_element` is given a type and it requires this
            // type implements `Deserialize`, this is how it achieves the deserialization for non-primitive
            // types.
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let zero = match SeqAccess::next_element::<String>(&mut seq)? {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(0usize, &"struct Dog with 3 elements"));
                    }
                };

                let one = match SeqAccess::next_element::<u8>(&mut seq)? {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(1usize, &"struct Dog with 3 elements"));
                    }
                };

                // `next_element` will invoke `Breed::deserialize` and pass it the deserializer,
                // (stored in SeqAccess) which will then consume the next few bytes and result in
                // the Breed enum variant.
                let two = match SeqAccess::next_element::<Breed>(&mut seq)? {
                    Some(value) => value,
                    None => {
                        return Err(Error::invalid_length(2usize, &"struct Dog with 3 elements"));
                    }
                };
                Ok(Dog {
                    name: zero,
                    age: one,
                    breed: two,
                })
            }

            // 2. This method is invoked by the deseriazer if it sees a indicator like `{` in its data stream.
            // this method knows about the fields (fields can be deserialized using the `FieldVisitor` above
            // in a number of ways), and will request the right data types from the deserializer using this
            // knowledge.
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut zero: Option<String> = None;
                let mut one: Option<u8> = None;
                let mut two: Option<Breed> = None;
                // 3. `next_key` will use `Field::deserialize` which uses `FieldVisitor` to
                // deserialize the key.
                while let Some(key) = MapAccess::next_key::<Field>(&mut map)? {
                    match key {
                        Field::Zero => {
                            if Option::is_some(&zero) {
                                return Err(<A::Error as Error>::duplicate_field("name"));
                            }
                            zero = Some(MapAccess::next_value::<String>(&mut map)?);
                        }
                        Field::One => {
                            if Option::is_some(&one) {
                                return Err(<A::Error as Error>::duplicate_field("age"));
                            }
                            one = Some(MapAccess::next_value::<u8>(&mut map)?);
                        }
                        Field::Two => {
                            if Option::is_some(&two) {
                                return Err(<A::Error as Error>::duplicate_field("breed"));
                            }

                            // `next_element` will invoke `Breed::deserialize` and pass it the deserializer,
                            // (stored in SeqAccess) which will then consume the next few bytes and result in
                            // the Breed enum variant.
                            two = Some(MapAccess::next_value::<Breed>(&mut map)?);
                        }
                        _ => {
                            let _ = MapAccess::next_value::<IgnoredAny>(&mut map)?;
                        }
                    }
                }
                let zero = match zero {
                    Some(zero) => zero,
                    None => serde::__private::de::missing_field("name")?,
                };

                let one = match one {
                    Some(one) => one,
                    None => serde::__private::de::missing_field("age")?,
                };

                let two = match two {
                    Some(two) => two,
                    None => serde::__private::de::missing_field("breed")?,
                };

                Ok(Dog {
                    name: zero,
                    age: one,
                    breed: two,
                })
            }
        }

        // 1. It all starts here. All the impl blocks above are part of the method but what's
        // below this point is what's actually executed. Now, what does `deserialize_struct` do?
        // See this link: https://github.com/serde-rs/json/blob/1e77cac742aaa12d0c8390bd8d40e279e05a3bca/src/de.rs#L1812.
        // In essence, it checks the input for an array or object, then calls `visit_seq` or `visit_map`
        // respectively on the `DogVisitor`. See these methods for step 2.


        const FIELDS: &[&str] = &["name", "age", "breed"];

        Deserializer::deserialize_struct(
            deserializer,
            "Dog",
            FIELDS,
            DogVisitor {
                marker: PhantomData,
                lifetime: PhantomData,
            },
        )
    }
}

enum Breed {
    Husky,
    #[allow(dead_code)]
    Teckel,
}

impl<'de> Deserialize<'de> for Breed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Zero,
            One,
        }

        struct FieldVisitor;

        // Here again we have 3 ways implemented to deserialize a enum to a "field"
        // but notice here the return type of the visitor is the index that the variant
        // has in the enum definition.
        impl Visitor<'_> for FieldVisitor {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                Formatter::write_str(formatter, "variant identifier")
            }

            fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    0u64 => Ok(Field::Zero),
                    1u64 => Ok(Field::One),
                    _ => Err(Error::invalid_value(
                        Unexpected::Unsigned(value),
                        &"variant index 0 <= i < 2",
                    )),
                }
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    "Husky" => Ok(Field::Zero),
                    "Teckel" => Ok(Field::One),
                    _ => Err(Error::unknown_variant(value, VARIANTS)),
                }
            }

            fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
            where
                E: Error,
            {
                match value {
                    b"Husky" => Ok(Field::Zero),
                    b"Teckel" => Ok(Field::One),
                    _ => {
                        let value = &String::from_utf8_lossy(value);
                        Err(Error::unknown_variant(value, VARIANTS))
                    }
                }
            }
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                // `deserialize_identifier` here attempts to deserialize a string (the field)
                Deserializer::deserialize_identifier(deserializer, FieldVisitor)
            }
        }

        struct BreedVisitor<'de> {
            marker: PhantomData<Breed>,
            lifetime: PhantomData<&'de ()>,
        }

        impl<'de> Visitor<'de> for BreedVisitor<'de> {
            type Value = Breed;
            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                Formatter::write_str(formatter, "enum Breed")
            }

            // This is the method that is called to define which variant belongs to which index.

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                // `data.variant()` here infers the type of the "field" to be `Field`,
                // which implements `Deserialize` and so it uses that implementation to
                // deserialize the field into the `Field` enum.
                match EnumAccess::variant(data)? {
                    // `variant` here is of type `VariantAccess` which we can use to access data stored
                    // by the variant: https://github.com/serde-rs/json/blob/1e77cac742aaa12d0c8390bd8d40e279e05a3bca/src/de.rs#L2041
                    (Field::Zero, variant) => {
                        // Then we say here, for `Field::Zero` the variant must be a unit variant.
                        // (it must hold no data) which is checked by the `variant.unit_variant()` method.
                        VariantAccess::unit_variant(variant)?;
                        Ok(Breed::Husky)
                    }
                    (Field::One, variant) => {
                        VariantAccess::unit_variant(variant)?;
                        Ok(Breed::Teckel)
                    }
                }
            }
        }

        const VARIANTS: &[&str] = &["Husky", "Teckel"];

        // 5. `DogVisitor::visit_(seq/map)` will end up invoking this method.
        Deserializer::deserialize_enum(
            deserializer,
            "Breed",
            VARIANTS,
            BreedVisitor {
                marker: PhantomData,
                lifetime: PhantomData,
            },
        )
    }
}
```

This one is all about the visitor pattern really, a pattern that I found really hard to grasp. So let's walk through the execution here step by step:

1. `deserialize` method is called by `serde_json`, passing in its `Deserializer` struct
2. This will call `Deserializer::deserialize_struct`, passing in a `Visitor` implementation specifically for the struct type.
3. A `Field` enum is created with a `Visitor` implementation capable of deserializing several data formats into the "keys/fields" of the struct.
4. The `deserialize_struct` method will deserialize the "opening" of the struct in the serialized data format (`{` in JSON)
5. Next it will call `visit_map` on the passed in `Visitor`, which is implemented specifically for the struct type, and thus knows about the keys and values. It will request these from the `Deserializer` using the `MapAccess` object passed as an argument, which drives the deserializer to pull these values from the serialized data.
6. `visit_map` will return the struct with the values and keys, meaning deserialization is complete!

For enums it's not much different, here's what changes:

- `FieldVisitor` checks for 3 serialized data formats for the keys, but these might also just be the name of the variant as with enums there's no key/value concept.
- The `Visitor` implements `visit_enum` which matches on the `Field` to identify the variant, then `EnumAccess::variant(data)` returns a `VariantAccess` interface which is used to deserialize the data potentially stored inside the enum.

## Resources

I learnt most of what's here by watching [this really good deep-dive by `@jonhoo`](https://youtu.be/BI_bHCGRgMY?si=SMTYYwogmbVQv9JC). You really ought to watch it to reinforce these concepts.

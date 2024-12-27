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

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut zero: Option<String> = None;
                let mut one: Option<u8> = None;
                let mut two: Option<Breed> = None;
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

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
            where
                A: EnumAccess<'de>,
            {
                match EnumAccess::variant(data)? {
                    (Field::Zero, variant) => {
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

# seriously

How does the serde crate actually work? Stick around to find out!

## cargo-expand

I've used `cargo-expand to unpack all the derive macros in use, and prettified what code was generated. Take a look for yourself!

- [deserialize.rs](./src/bin/deserialize.rs)
- [deserialize_expanded.rs](./src/bin/deserialize_expanded.rs)

- [serialize.rs](./src/bin/serialize.rs)
- [serialize_expanded.rs](./src/bin/serialize_expanded.rs)

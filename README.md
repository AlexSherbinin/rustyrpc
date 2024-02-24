# RustyRPC

RustyRPC is a Rust RPC framework designed for simplicity and ease of use. Defining a service requires just a few lines of code, and much of the server boilerplate is handled automatically.

### Features of Rustyrpc

RustyRPC distinguishes itself by defining schemas in code, avoiding a separate compilation process and language switching. Key features include:

 - **Transport Agnostic**: Choose any transport, from HTTP/2 to TCP (currently only QUIC is supported).

 - **Encoding Format Agnostic**: Choose any format, such as JSON, Cap'n Proto, or rkyv (currently only rkyv is supported).

 - **Object-Oriented**: You can return a service from a function of a service!

## License

This crate is licensed under the [MIT License](LICENSE).

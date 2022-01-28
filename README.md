# Rust sample web server

This application was built on top of the [last chapter](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html) of the learning rust book.

Once this application starts it will fetch from Yahoo finance quote price information for stocks with symbols defined at `symbols.txt`.

## Practiced learning concepts

- Generics
- Structs
- Pattern matching
- Third party crates
- Unit tests
- Module separation
- Sharing context among threads (HashMap)

## Environment setup

Ensure openssl dev headers are available:

```bash
$ sudo apt-get install libssl-dev
```

Run it with `cargo run`

Issue a `GET` request to retrieve the available symbols:

```bash
$ curl -XGET http://127.0.0.1:7878/api/v1/quotes
```

Use one of the symbos and issue a second request:

```bash
$ curl -XGET http://127.0.0.1:7878/api/v1/quotes/MSFT
```
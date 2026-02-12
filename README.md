# httpie
A native httpie implementation with Rust.

# Usage
```bash
cargo build .

> ./target/debug/httpie --help
A native httpie implementation with Rust, can you imagine how easy it is?

Author: Bourne <bourne@proton.me>

Usage: httpie <COMMAND>

Commands:
  get
  post
  help  Print this message or the help of the given subcommand(s)

Options:
  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version

> ./target/debug/httpie post https://httpbin.org/post a=b c=d
HTTP/2.0 200 OK

date: "Thu, 12 Feb 2026 12:34:59 GMT"
content-type: "application/json"
content-length: "477"
server: "gunicorn/19.9.0"
access-control-allow-origin: "*"
access-control-allow-credentials: "true"

{
  "args": {},
  "data": "{\"a\":\"b\",\"c\":\"d\"}",
  "files": {},
  "form": {},
  "headers": {
    "Accept": "*/*",
    "Content-Length": "17",
    "Content-Type": "application/json",
    "Host": "httpbin.org",
    "User-Agent": "httpie-rust/0.1.0",
    "X-Amzn-Trace-Id": "Root=1-698dc8f3-1c9e64cc0737e63b7aab799e",
    "X-Power-By": "Rust"
  },
  "json": {
    "a": "b",
    "c": "d"
  },
  "url": "https://httpbin.org/post"
}
```
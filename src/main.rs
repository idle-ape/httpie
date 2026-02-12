use core::str;
use std::{collections::HashMap, str::FromStr};

use anyhow::{Ok, Result};
use clap::{Args, Parser, Subcommand};
use colored::*;
use mime::Mime;
use reqwest::{Client, Response, Url, header};
use syntect::{
    easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet, util::LinesWithEndings,
};

#[derive(Subcommand, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

#[derive(Debug, Args)]
struct Get {
    #[clap(value_parser = parse_url)]
    url: String,
}

#[derive(Debug, Args)]
struct Post {
    #[clap(value_parser = parse_url)]
    url: String,
    #[clap(value_parser = parse_kv_pair)]
    body: Vec<KvPair>,
}

fn parse_url(s: &str) -> Result<String> {
    // A simple URL validation (you can use a more robust solution)
    let _url = s.parse::<Url>()?;
    Ok(s.into())
}

#[derive(Debug, Clone, PartialEq)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.splitn(2, '=');
        let k = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("No key found"))?
            .to_string();
        let v = split
            .next()
            .ok_or_else(|| anyhow::anyhow!("No value found"))?
            .to_string();
        Ok(KvPair { k, v })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

#[derive(Parser, Debug)]
#[command(
    name = "httpie",
    version = "0.1.0",
    author = "Bourne <bourne@proton.me>",
    about = "A native httpie implementation with Rust | Author: Bourne <bourne@proton.me>",
    long_about = "A native httpie implementation with Rust, can you imagine how easy it is?\n\nAuthor: Bourne <bourne@proton.me>",
    long_version = "httpie 0.1.0\nAuthor: Bourne <bourne@proton.me>"
)]
struct Opts {
    #[command(subcommand)]
    subcmd: SubCommand,
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts = Opts::parse();
    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWER-BY", "Rust".parse()?);
    headers.insert(
        header::USER_AGENT,
        header::HeaderValue::from_static("httpie-rust/0.1.0"),
    );
    let client = Client::builder().default_headers(headers).build()?;
    match opts.subcmd {
        SubCommand::Get(args) => get(client, &args).await?,
        SubCommand::Post(body) => post(client, &body).await?,
    };
    Ok(())
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, post: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in post.body.iter() {
        body.insert(&pair.k, &pair.v);
    }

    let resp = client.post(&post.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_header(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    print_body(mime, &body);
    Ok(())
}

fn print_status(resp: &Response) {
    // 注意看 Colorize trait 的定义，它的方法 consume 的都是 self，而非 &self。所以当 impl Colorize for &str 时，self = &str。
    // 在调用方法时，编译器会先看数据结构是否有对应的方法，如果有，按照方法的 signature，传 self / &self / &mut self。如果没有，再看引入的 trait 是否有对应的方法，必要时会根据 self 的类型做 auto Deref。所以这里编译器可以找到 blue()，因为它第一个参数 self = &str，String 可以 Deref 到 &str，所以可以调用。但如果 Colorize 的方法使用 &self，此时 &self = &&str，String 无法 Deref 到 &&str，所以编译器报错。
    // 你可以看这个小例子：
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=dbd526df998fc8701ea9855c9e7de73c
    //
    // pub trait Foo {
    //    fn foo(self);
    // }
    //
    // pub trait Bar {
    //    fn bar(&self);
    // }
    //
    // pub trait Baz {
    //     fn baz(&self);
    // }
    //
    // impl Foo for &str {
    //    fn foo(self) {
    //        println!("Foo: {}", self);
    //    }
    // }
    //
    // impl Bar for &str {
    //    fn bar(&self) {
    //        println!("Bar: {}", self);
    //    }
    // }
    // 
    // impl Baz for str {
    //    fn baz(&self) {
    //        println!("Baz: {}", self);
    //    }
    // }
    //
    // fn main() {
    //    let s = String::from("Tyr");
    //    // foo 第一个参数是 self = &str，String 可以 auto Deref 到 &str，所以可以调用
    //    s.foo();
    //    // bar 第一个参数是 &self = &&str，String 无法 auto Deref 到 &&str
    //    // s.bar();
    //    // baz 第一个参数是 &self，但因为 impl Baz for str {}, 所以 &self = &str
    //    // 和 foo 类似，可以调用
    //    s.baz();
    // }
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{status}\n");
}

fn print_header(resp: &Response) {
    let header = resp.headers();
    for (k, v) in header.iter() {
        println!("{}: {:?}", k.to_string().green(), v);
    }
    println!();
}

fn print_body(mime: Option<Mime>, body: &String) {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let syntax = ps.find_syntax_by_name("JSON").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    match mime {
        Some(m) if m == mime::APPLICATION_JSON => {
            let json = jsonxf::pretty_print(body).unwrap();
            for line in LinesWithEndings::from(&json) {
                let ranges = h.highlight_line(line, &ps).unwrap();
                let escaped = syntect::util::as_24_bit_terminal_escaped(&ranges[..], false);
                println!("{escaped}");
            }
        }
        _ => println!("{body}"),
    }
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_url_works() {
        assert!(parse_url("https://httpbin.org").is_ok());
        assert!(parse_url("httpbin.org").is_err());
    }

    #[test]
    fn parse_kv_pair_works() {
        assert!(parse_kv_pair("a").is_err());
        assert!(parse_kv_pair("a=b").is_ok());
        assert_eq!(
            parse_kv_pair("a=b").unwrap(),
            KvPair {
                k: "a".into(),
                v: "b".into(),
            }
        );
    }
}

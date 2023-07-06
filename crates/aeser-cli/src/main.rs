use clap::Parser;
use aeser::api_encoder::{decode_check, decode_id, KnownType};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    encoded_string: String,
}

fn main() {
    let cli = Cli::parse();
    println!("CLI: {:?}", cli);
    let kt = KnownType::AccountPubkey;
    println!("prefix: {:?}", kt.prefix());
    let dec = decode_id(&[kt], &cli.encoded_string);
    println!("decoded: {:?}", dec);
}
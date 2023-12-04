mod sled_oracle;

fn main() {
    let oracle = sled_oracle::SledOracle::new("oracle_db").unwrap();
    println!("Hello, world!");
}

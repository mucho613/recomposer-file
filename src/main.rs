mod header_block;

fn main() {
    let rcp_file = std::fs::read("yuno35pr.rcp").expect("Failed to read file");
    let rcp = header_block::parse::load(&rcp_file).expect("Failed to parse RCP");
}

mod header_block;

fn main() {
    let rcp_file = std::fs::read("test.rcp").expect("Failed to read file");
    header_block::parse::load(&rcp_file).expect("Failed to parse RCP");
}

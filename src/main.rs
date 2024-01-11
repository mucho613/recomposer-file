use header_block::types::HeaderBlock;
use track_block::types::TrackBlock;

mod header_block;
mod track_block;

#[derive(Debug)]
pub struct RcpFile {
    pub header_block: HeaderBlock,
    pub track_block: TrackBlock,
}

fn main() {
    let rcp_file = std::fs::read("test.rcp").expect("Failed to read file");

    let (i, header_block) =
        header_block::parse::parse_header_block(&rcp_file).expect("Failed to parse RCP");

    let (_, track_block) = track_block::parse::parse_track_block(i).expect("Failed to parse RCP");

    let rcp = RcpFile {
        header_block,
        track_block,
    };
}

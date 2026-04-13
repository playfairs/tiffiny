const TIFF_HEADER: &[u8] = b"II*\0";
const TIFF_OFFSET: &[u8] = &[0u8; 8];
const TIFF_FOOTER: &[u8] = b"TIFFINY";

pub fn inject_headers(raw_audio: &[u8]) -> Vec<u8> {
    let mut buffer = Vec::new();
    buffer.extend_from_slice(TIFF_HEADER);
    buffer.extend_from_slice(TIFF_OFFSET);
    buffer.extend_from_slice(raw_audio);
    buffer.extend_from_slice(TIFF_FOOTER);
    buffer
}

pub fn extract_audio(tiff_data: &[u8]) -> Vec<u8> {
    let start = TIFF_HEADER.len() + TIFF_OFFSET.len();
    let end = tiff_data.len().saturating_sub(TIFF_FOOTER.len());
    
    if start < end {
        tiff_data[start..end].to_vec()
    } else {
        Vec::new()
    }
}

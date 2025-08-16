pub struct RawImage(pub u32, pub u32, pub Vec<u8>);

impl RawImage {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.0.to_be_bytes());
        bytes.extend_from_slice(&self.1.to_be_bytes());
        bytes.extend_from_slice(&self.2);
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let width = u32::from_be_bytes(bytes[0..4].try_into().unwrap());
        let height = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
        if bytes.len() != width as usize * height as usize * 4 + 8 {
            return Err("Image dimensions conflict with byte stream length".into());
        }
        let data = bytes[8..width as usize * height as usize * 4].to_vec();
        Ok(RawImage(width, height, data))
    }
}

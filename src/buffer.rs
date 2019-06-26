pub const BUFFER_SIZE: usize = 4096;

pub type Buffer = [u8; BUFFER_SIZE];

pub fn from_vec(bytes: Vec<u8>) -> Buffer {
    let mut buf: Buffer = [0; BUFFER_SIZE];
    for (place, byte) in buf.iter_mut().zip(bytes.iter()) {
        *place = *byte;
    }

    buf
}

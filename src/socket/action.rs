use std::io::Error;

pub trait SocketAction {
    fn disconnect(&mut self) -> Result<(), Error>;
    fn listen(&self, buffer_size: usize) -> Result<Vec<u8>, Error>;
    fn send(&self, buffer_size: &[u8]) -> std::io::Result<usize>;
}

use std::sync::MutexGuard;
use std::io::Read;
use std::fs::File;

pub struct MutexReader<'a>(pub MutexGuard<'a, File>);

impl<'a> Read for MutexReader<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.0.read(buf)
    }
}
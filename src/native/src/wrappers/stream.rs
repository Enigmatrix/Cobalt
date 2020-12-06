use crate::error::*;
use std::io::{Read, Seek, SeekFrom};

mod windows {
    pub use crate::raw::uwp::windows::*;
}

use windows::security::cryptography::CryptographicBuffer;
use windows::storage::streams::*;

#[derive(Debug, Clone)]
pub struct WinRTImageStream {
    stream: IRandomAccessStreamWithContentType,
}

impl WinRTImageStream {
    fn read_from_buffer(buffer: &IBuffer, buf: &mut [u8]) -> ::winrt::Result<usize> {
        let sz = buffer.length().unwrap() as usize;
        DataReader::from_buffer(buffer)
            .unwrap()
            .read_bytes(&mut buf[..sz])?;
        Ok(sz)
    }
}

impl<T: Into<IRandomAccessStreamWithContentType>> From<T> for WinRTImageStream {
    fn from(stream: T) -> Self {
        WinRTImageStream {
            stream: stream.into(),
        }
    }
}

impl Read for WinRTImageStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let buffer = CryptographicBuffer::create_from_byte_array(buf).unwrap();
        let written = self
            .stream
            .read_async(buffer, buf.len() as u32, InputStreamOptions::None)
            .to_std()?
            .get()
            .to_std()?;

        let written_sz = WinRTImageStream::read_from_buffer(&written, buf).to_std()?;

        Ok(written_sz)
    }
}

impl Seek for WinRTImageStream {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        let pos = match pos {
            SeekFrom::Start(pos) => pos,
            SeekFrom::End(pos) => {
                let size = self.stream_len()?;
                size - 1 - pos as u64
            }
            SeekFrom::Current(pos) => (self.stream_position()? as i64 + pos) as u64,
        };
        self.stream.seek(pos).to_std()?;
        Ok(pos)
    }

    fn stream_len(&mut self) -> std::io::Result<u64> {
        self.stream.size().to_std()
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.stream.position().to_std()
    }
}

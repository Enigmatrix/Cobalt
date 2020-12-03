use crate::error::WinRt;
use std::io::{Read, Seek, SeekFrom};
use util::*;

mod windows {
    pub use crate::raw::uwp::windows::*;
}

use windows::storage::streams::*;

pub struct WinRTImageStream {
    stream: IRandomAccessStreamWithContentType,
}

impl WinRTImageStream {
    fn read_from_buffer(buffer: &IBuffer, buf: &mut [u8]) -> ::winrt::Result<usize> {
        let sz = buffer.length()? as usize;
        DataReader::from_buffer(buffer)?.read_bytes(&mut buf[..sz])?;
        Ok(sz)
    }
}

fn wait_complete<T: ::winrt::RuntimeType, R: ::winrt::RuntimeType>(op: windows::foundation::IAsyncOperationWithProgress<T, R>) -> ::winrt::Result<T> {
    if op.status()? == ::winrt::foundation::AsyncStatus::Started {
        let (sender,recver) = std::sync::mpsc::sync_channel(1);
        op.set_completed(::winrt::foundation::AsyncOperationWithProgressCompletedHandler::new(move |_sender, _args| {
            sender.send(());
            Ok(())
        }))?;
        recver.recv().unwrap();
    }
    op.get_results()
}

impl From<IRandomAccessStreamWithContentType> for WinRTImageStream {
    fn from(stream: IRandomAccessStreamWithContentType) -> Self {
        WinRTImageStream { stream }
    }
}

impl Read for WinRTImageStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let buffer = windows::security::cryptography::CryptographicBuffer::create_from_byte_array(buf).unwrap();
        let ev = self
            .stream
            .read_async(
                IBuffer::from(buffer),
                buf.len() as u32,
                InputStreamOptions::None,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, WinRt::from(e)))?; // TODO use better conversion
            
        let written = wait_complete(ev)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, WinRt::from(e)))?; // TODO use better conversion

        let written_sz = WinRTImageStream::read_from_buffer(&written, buf) 
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, WinRt::from(e)))?; // TODO use better conversion

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
        self.stream
            .seek(pos)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, WinRt::from(e)))?; // TODO use better conversion
        Ok(pos)
    }

    fn stream_len(&mut self) -> std::io::Result<u64> {
        self.stream
            .size()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, WinRt::from(e)))
        // TODO use better conversion
    }

    fn stream_position(&mut self) -> std::io::Result<u64> {
        self.stream
            .position()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, WinRt::from(e)))
        // TODO use better conversion
    }
}

#[::winrt::implement(windows::storage::streams::IBuffer)]
pub struct ByteBuffer {
    _ptr: *mut u8,
    len: std::cell::Cell<u32>,
    capacity: u32,
}

impl ByteBuffer {
    pub fn new(bytes: &mut [u8]) -> Self {
        let _ptr = bytes.as_mut_ptr();
        let capacity = bytes.len() as u32;
        ByteBuffer {
            _ptr,
            len: std::cell::Cell::new(0),
            capacity,
        }
    }

    pub fn capacity(&self) -> ::winrt::Result<u32> {
        Ok(self.capacity)
    }

    pub fn length(&self) -> ::winrt::Result<u32> {
        Ok(self.len.get())
    }
    pub fn set_length(&self, value: u32) -> ::winrt::Result<()> {
        self.len.set(value);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bytebuffer_to_ibuffer() {
        let bytes = &mut [123u8, 2, 3];
        let ibuffer = IBuffer::from(ByteBuffer::new(bytes));
        assert_eq!(0, ibuffer.length().unwrap());
        assert_eq!(3, ibuffer.capacity().unwrap());
        ibuffer.set_length(1).unwrap();
        assert_eq!(1, ibuffer.length().unwrap());
    }

    #[test]
    fn slice_to_ibuffer() {
        let bytes = &mut [123u8, 2, 3];
        let ibuffer = windows::security::cryptography::CryptographicBuffer::create_from_byte_array(&bytes[..]).unwrap();
        assert_eq!(0, ibuffer.length().unwrap());
        assert_eq!(3, ibuffer.capacity().unwrap());
        ibuffer.set_length(1).unwrap();
        assert_eq!(1, ibuffer.length().unwrap());
    }
}

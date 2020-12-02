use crate::error::WinRt;
use std::io::Read;
use util::*;

mod windows {
    pub use crate::raw::uwp::windows::*;
}

use windows::storage::streams::*;

pub struct WinRTStream {
    stream: IRandomAccessStream,
}

impl WinRTStream {
    pub fn size(&self) -> Result<u64> {
        Ok(self.stream.size().map_err(WinRt::from)?)
    }
}

impl<T: Into<IRandomAccessStream>> From<T> for WinRTStream {
    fn from(stream: T) -> Self {
        WinRTStream {
            stream: stream.into(),
        }
    }
}

impl Read for WinRTStream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let buffer = ByteBuffer::new(buf);
        let ev = self
            .stream
            .read_async(
                IBuffer::from(buffer),
                buf.len() as u32,
                InputStreamOptions::None,
            )
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, WinRt::from(e)))?;
        let written = ev
            .get()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, WinRt::from(e)))?;
        Ok(written.length().unwrap() as usize) // why would this even fail?
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

pub struct WinRTStreamToRustAdapter<'a> {
    stream: &'a IRandomAccessStreamWithContentType,
}

impl<'a> WinRTStreamToRustAdapter<'a> {
    pub fn from(stream: &'a IRandomAccessStreamWithContentType) -> WinRTStreamToRustAdapter<'a> {
        WinRTStreamToRustAdapter { stream }
    }

    // try benchamrking this with a `Read` implementation
    pub fn read_all(&self) -> Result<Vec<u8>> {
        let sz = self
            .stream
            .size()
            .map_err(WinRt::from)
            .with_context(|| "Get length of stream")?;
        let mut out = vec![0u8; sz as usize]; // TODO use uninit
        let reader = DataReader::create_data_reader(self.stream)
            .map_err(WinRt::from)
            .with_context(|| "Create DataReader")?;
        reader
            .load_async(sz as u32)
            .map_err(WinRt::from)
            .with_context(|| "Load data from stream")?
            .get()
            .map_err(WinRt::from)
            .with_context(|| "Run loading of data from stream in a blocking manner")?;
        reader
            .read_bytes(&mut out)
            .map_err(WinRt::from)
            .with_context(|| "Read bytes out of DataReader")?;
        Ok(out)
    }
}

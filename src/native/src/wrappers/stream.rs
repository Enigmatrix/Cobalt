use crate::error::WinRt;
use crate::raw::objidlbase::{
    ISequentialStream, ISequentialStreamVtbl, IStream, IStreamVtbl, STATSTG,
};
use crate::raw::unknwnbase::IUnknown;
use crate::raw::uwp::windows::storage::streams::*;
use crate::raw::*;
use anyhow::*;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;
use winapi::shared::guiddef::{IsEqualIID, REFIID};

const BUFFER_SIZE: usize = 4096;

pub struct HugeExtensibleBuffer {
    inner: Vec<u8>,
}

impl Default for HugeExtensibleBuffer {
    fn default() -> Self {
        HugeExtensibleBuffer::new()
    }
}

impl HugeExtensibleBuffer {
    pub fn new() -> HugeExtensibleBuffer {
        HugeExtensibleBuffer {
            inner: Vec::with_capacity(BUFFER_SIZE),
        }
    }

    pub fn consume(self) -> Vec<u8> {
        self.inner
    }
}

impl Write for HugeExtensibleBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let len = buf.len();
        let ptr = buf.as_ptr();

        let our_len = self.inner.len();
        let new_len = our_len + len;

        let needs_more = (new_len) as isize - self.inner.capacity() as isize;
        if needs_more >= 0 {
            self.inner.reserve(
                (((needs_more / BUFFER_SIZE as isize) + 1) * BUFFER_SIZE as isize) as usize,
            );
        }
        unsafe { std::ptr::copy_nonoverlapping(ptr, self.inner.as_mut_ptr().add(our_len), len) };
        unsafe { self.inner.set_len(new_len) }
        Ok(len)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub struct WinRTStream {
    stream: IRandomAccessStream
}

impl WinRTStream {
    pub fn size(&self) -> Result<u64> {
        Ok(self.stream.size().map_err(WinRt::from)?)
    }
}

impl<T: Into<IRandomAccessStream>> From<T> for WinRTStream {
    fn from(stream: T) -> Self {
        WinRTStream { stream: stream.into() }
    }
}

pub struct ByteBuffer<'a> {
    bytes: &'a mut [u8],
    len: u64
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

pub struct RustToWin32StreamAdapter<T> {
    vt: IStreamVtbl,
    inner: T,
}

impl<T> RustToWin32StreamAdapter<T> {
    pub fn from(inner: T) -> RustToWin32StreamAdapter<T> {
        RustToWin32StreamAdapter {
            inner,
            vt: default_stream(),
        }
    }

    pub fn as_istream(&self) -> IStream {
        IStream { lpVtbl: &self.vt }
    }

    pub fn inner(self) -> T {
        self.inner
    }

    unsafe fn get_inner<'a>(stream_ptr: *mut IStream) -> &'a mut T {
        let this: &mut RustToWin32StreamAdapter<T> = mem::transmute(stream_ptr.read());
        &mut this.inner
    }
}

impl<T: Write> RustToWin32StreamAdapter<T> {
    pub fn writeable(mut self) -> Self {
        self.vt.parent.Write = RustToWin32StreamAdapter::<T>::write;
        self
    }

    unsafe extern "system" fn write(
        this_ptr: *mut ISequentialStream,
        ptr: *const c_void,
        len: ULONG,
        out_len: *mut ULONG,
    ) -> HRESULT {
        let writer = RustToWin32StreamAdapter::<T>::get_inner(this_ptr.cast::<IStream>());
        let buffer = std::slice::from_raw_parts(ptr.cast::<u8>(), len as usize);
        io_result_to_hresult(writer.write(buffer), |val| *out_len = val as u32)
    }
}

impl<T: Read> RustToWin32StreamAdapter<T> {
    pub fn readable(mut self) -> Self {
        self.vt.parent.Read = RustToWin32StreamAdapter::<T>::read;
        self
    }

    unsafe extern "system" fn read(
        this_ptr: *mut ISequentialStream,
        ptr: *mut c_void,
        len: ULONG,
        out_len: *mut ULONG,
    ) -> HRESULT {
        let reader = RustToWin32StreamAdapter::<T>::get_inner(this_ptr.cast::<IStream>());
        let buffer = std::slice::from_raw_parts_mut(ptr.cast::<u8>(), len as usize);
        io_result_to_hresult(reader.read(buffer), |val| *out_len = val as u32)
    }
}

impl<T: Seek> RustToWin32StreamAdapter<T> {
    pub fn seekable(mut self) -> Self {
        self.vt.Seek = RustToWin32StreamAdapter::<T>::seek;
        self
    }

    unsafe extern "system" fn seek(
        this_ptr: *mut IStream,
        displacement: LARGE_INTEGER,
        origin: DWORD,
        pos: *mut ULARGE_INTEGER,
    ) -> HRESULT {
        let seeker = RustToWin32StreamAdapter::<T>::get_inner(this_ptr);
        match origin {
            objidlbase::STREAM_SEEK_SET => {
                let pos = pos.cast::<u64>();
                io_result_to_hresult(
                    seeker.seek(SeekFrom::Start(*displacement.QuadPart() as u64)),
                    |val| *pos = val,
                )
            }
            _ => unreachable!(),
        }
    }
}

#[inline(always)]
fn io_result_to_hresult<T>(res: std::io::Result<T>, success: impl Fn(T)) -> HRESULT {
    match res {
        Ok(value) => {
            success(value);
            winerror::S_OK
        }
        Err(e) => match e.raw_os_error() {
            Some(code) => code,
            None => winerror::E_FAIL,
        },
    }
}

#[allow(non_snake_case)]
#[allow(unused_variables)]
fn default_stream() -> IStreamVtbl {
    unsafe extern "system" fn QueryInterface(
        This: *mut IUnknown,
        riid: REFIID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT {
        let riid = &*riid;
        if IsEqualIID(riid, &IStream::uuidof())
            || IsEqualIID(riid, &ISequentialStream::uuidof())
            || IsEqualIID(riid, &IUnknown::uuidof())
        {
            *ppvObject = This.cast();
            winerror::S_OK
        } else {
            winerror::E_NOINTERFACE
        }
    }
    unsafe extern "system" fn AddRef(This: *mut IUnknown) -> ULONG {
        1
    }
    unsafe extern "system" fn Release(This: *mut IUnknown) -> ULONG {
        1
    }
    unsafe extern "system" fn Read(
        This: *mut ISequentialStream,
        pv: *mut c_void,
        cb: ULONG,
        pcbRead: *mut ULONG,
    ) -> HRESULT {
        unimplemented!("Read")
    }
    unsafe extern "system" fn Write(
        This: *mut ISequentialStream,
        pv: *const c_void,
        cb: ULONG,
        pcbWritten: *mut ULONG,
    ) -> HRESULT {
        unimplemented!("Write")
    }
    unsafe extern "system" fn Seek(
        This: *mut IStream,
        dlibMove: LARGE_INTEGER,
        dwOrigin: DWORD,
        plibNewPosition: *mut ULARGE_INTEGER,
    ) -> HRESULT {
        unimplemented!("Seek")
    }
    unsafe extern "system" fn SetSize(This: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT {
        unimplemented!("SetSize")
    }
    unsafe extern "system" fn CopyTo(
        This: *mut IStream,
        pstm: *mut IStream,
        cb: ULARGE_INTEGER,
        pcbRead: *mut ULARGE_INTEGER,
        pcbWritten: *mut ULARGE_INTEGER,
    ) -> HRESULT {
        unimplemented!("CopyTo")
    }
    unsafe extern "system" fn Commit(This: *mut IStream, grfCommitFlags: DWORD) -> HRESULT {
        tracing::warn!("COMMIT!!");
        unimplemented!("Commit")
    }
    unsafe extern "system" fn Revert(This: *mut IStream) -> HRESULT {
        unimplemented!("Revert")
    }
    unsafe extern "system" fn LockRegion(
        This: *mut IStream,
        libOffset: ULARGE_INTEGER,
        cb: ULARGE_INTEGER,
        dwLockType: DWORD,
    ) -> HRESULT {
        unimplemented!("LockRegion")
    }
    unsafe extern "system" fn UnlockRegion(
        This: *mut IStream,
        libOffset: ULARGE_INTEGER,
        cb: ULARGE_INTEGER,
        dwLockType: DWORD,
    ) -> HRESULT {
        unimplemented!("UnlockRegion")
    }
    unsafe extern "system" fn Stat(
        This: *mut IStream,
        pstatstg: *mut STATSTG,
        grfStatFlag: DWORD,
    ) -> HRESULT {
        unimplemented!("Stat")
    }
    unsafe extern "system" fn Clone(This: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT {
        unimplemented!("Clone")
    }

    IStreamVtbl {
        parent: ISequentialStreamVtbl {
            parent: unknwnbase::IUnknownVtbl {
                QueryInterface,
                AddRef,
                Release,
            },
            Read,
            Write,
        },
        Seek,
        SetSize,
        CopyTo,
        Commit,
        Revert,
        LockRegion,
        UnlockRegion,
        Stat,
        Clone,
    }
}

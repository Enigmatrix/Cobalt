use crate::raw::objidlbase::{
    ISequentialStream, ISequentialStreamVtbl, IStream, IStreamVtbl, STATSTG,
};
use crate::raw::unknwnbase::IUnknown;
use crate::raw::*;
use std::io::{Read, Seek, SeekFrom, Write};
use std::mem;
use winapi::shared::guiddef::{IsEqualIID, REFIID};

const BUFFER_SIZE: usize = 4096;

pub struct HugeExtensibleBuffer {
    inner: Vec<u8>,
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

pub struct Stream<T> {
    vt: IStreamVtbl,
    inner: T,
}

impl<T> Stream<T> {
    pub fn from(inner: T) -> Stream<T> {
        Stream {
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
        let this: &mut Stream<T> = mem::transmute(stream_ptr.read());
        &mut this.inner
    }
}

impl<T: Write> Stream<T> {
    pub fn writeable(mut self) -> Self {
        self.vt.parent.Write = Stream::<T>::write;
        self
    }

    unsafe extern "system" fn write(
        this_ptr: *mut ISequentialStream,
        ptr: *const c_void,
        len: ULONG,
        out_len: *mut ULONG,
    ) -> HRESULT {
        let writer = Stream::<T>::get_inner(this_ptr.cast::<IStream>());
        let buffer = std::slice::from_raw_parts(ptr.cast::<u8>(), len as usize);
        io_result_to_hresult(writer.write(buffer), |val| *out_len = val as u32)
    }
}

impl<T: Read> Stream<T> {
    pub fn readable(mut self) -> Self {
        self.vt.parent.Read = Stream::<T>::read;
        self
    }

    unsafe extern "system" fn read(
        this_ptr: *mut ISequentialStream,
        ptr: *mut c_void,
        len: ULONG,
        out_len: *mut ULONG,
    ) -> HRESULT {
        let reader = Stream::<T>::get_inner(this_ptr.cast::<IStream>());
        let buffer = std::slice::from_raw_parts_mut(ptr.cast::<u8>(), len as usize);
        io_result_to_hresult(reader.read(buffer), |val| *out_len = val as u32)
    }
}

impl<T: Seek> Stream<T> {
    pub fn seekable(mut self) -> Self {
        self.vt.Seek = Stream::<T>::seek;
        self
    }

    unsafe extern "system" fn seek(
        this_ptr: *mut IStream,
        displacement: LARGE_INTEGER,
        origin: DWORD,
        pos: *mut ULARGE_INTEGER,
    ) -> HRESULT {
        let seeker = Stream::<T>::get_inner(this_ptr);
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
        todo!("Read")
    }
    unsafe extern "system" fn Write(
        This: *mut ISequentialStream,
        pv: *const c_void,
        cb: ULONG,
        pcbWritten: *mut ULONG,
    ) -> HRESULT {
        todo!("Write")
    }
    unsafe extern "system" fn Seek(
        This: *mut IStream,
        dlibMove: LARGE_INTEGER,
        dwOrigin: DWORD,
        plibNewPosition: *mut ULARGE_INTEGER,
    ) -> HRESULT {
        todo!("Seek")
    }
    unsafe extern "system" fn SetSize(This: *mut IStream, libNewSize: ULARGE_INTEGER) -> HRESULT {
        todo!("SetSize")
    }
    unsafe extern "system" fn CopyTo(
        This: *mut IStream,
        pstm: *mut IStream,
        cb: ULARGE_INTEGER,
        pcbRead: *mut ULARGE_INTEGER,
        pcbWritten: *mut ULARGE_INTEGER,
    ) -> HRESULT {
        todo!("CopyTo")
    }
    unsafe extern "system" fn Commit(This: *mut IStream, grfCommitFlags: DWORD) -> HRESULT {
        tracing::warn!("COMMIT!!");
        todo!("Commit")
    }
    unsafe extern "system" fn Revert(This: *mut IStream) -> HRESULT {
        todo!("Revert")
    }
    unsafe extern "system" fn LockRegion(
        This: *mut IStream,
        libOffset: ULARGE_INTEGER,
        cb: ULARGE_INTEGER,
        dwLockType: DWORD,
    ) -> HRESULT {
        todo!("LockRegion")
    }
    unsafe extern "system" fn UnlockRegion(
        This: *mut IStream,
        libOffset: ULARGE_INTEGER,
        cb: ULARGE_INTEGER,
        dwLockType: DWORD,
    ) -> HRESULT {
        todo!("UnlockRegion")
    }
    unsafe extern "system" fn Stat(
        This: *mut IStream,
        pstatstg: *mut STATSTG,
        grfStatFlag: DWORD,
    ) -> HRESULT {
        todo!("Stat")
    }
    unsafe extern "system" fn Clone(This: *mut IStream, ppstm: *mut *mut IStream) -> HRESULT {
        todo!("Clone")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_works() {
        let wut = Stream::from(1);
        unsafe {
            // wut.stream().Revert();
        }
    }
}

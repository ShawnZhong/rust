use crate::io::{self, BorrowedCursor, IoSlice, IoSliceMut};

pub struct Stdin;
pub struct Stdout;
pub struct Stderr;

// verus-explorer: Miri-side stdout/stderr hooks. When this libstd build
// runs under a Miri instance compiled by `verus-explorer`, every
// `print!` / `println!` / `eprint!` / `eprintln!` call lands here and
// forwards bytes to Miri's `miri_write_to_stdout` / `miri_write_to_stderr`
// shims (see `third_party/rust/src/tools/miri/src/shims/foreign_items.rs`).
// Those shims call host `io::stdout()` / `io::stderr()`, which the
// `verus-explorer` patches in `shims/files.rs` route to the
// `__verus_explorer_stdout/stderr` externs in our wasm crate. Gated on
// `cfg(verus_explorer)` (set by `RUSTFLAGS_NOT_BOOTSTRAP` in
// `scripts/build-libs-sysroot.sh`) so other consumers of this
// fallback aren't affected.
#[cfg(verus_explorer)]
unsafe extern "Rust" {
    fn miri_write_to_stdout(buf: &[u8]);
    fn miri_write_to_stderr(buf: &[u8]);
}

impl Stdin {
    pub const fn new() -> Stdin {
        Stdin
    }
}

impl io::Read for Stdin {
    #[inline]
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn read_buf(&mut self, _cursor: BorrowedCursor<'_>) -> io::Result<()> {
        Ok(())
    }

    #[inline]
    fn read_vectored(&mut self, _bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn is_read_vectored(&self) -> bool {
        // Do not force `Chain<Empty, T>` or `Chain<T, Empty>` to use vectored
        // reads, unless the other reader is vectored.
        false
    }

    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> io::Result<()> {
        if !buf.is_empty() { Err(io::Error::READ_EXACT_EOF) } else { Ok(()) }
    }

    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> io::Result<()> {
        if cursor.capacity() != 0 { Err(io::Error::READ_EXACT_EOF) } else { Ok(()) }
    }

    #[inline]
    fn read_to_end(&mut self, _buf: &mut Vec<u8>) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn read_to_string(&mut self, _buf: &mut String) -> io::Result<usize> {
        Ok(0)
    }
}

impl Stdout {
    pub const fn new() -> Stdout {
        Stdout
    }
}

impl io::Write for Stdout {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        #[cfg(verus_explorer)]
        unsafe { miri_write_to_stdout(buf) };
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut total_len = 0;
        for buf in bufs {
            #[cfg(verus_explorer)]
            unsafe { miri_write_to_stdout(buf) };
            total_len += buf.len();
        }
        Ok(total_len)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, _buf: &[u8]) -> io::Result<()> {
        #[cfg(verus_explorer)]
        unsafe { miri_write_to_stdout(_buf) };
        Ok(())
    }

    #[inline]
    fn write_all_vectored(&mut self, _bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        for _buf in _bufs.iter() {
            #[cfg(verus_explorer)]
            unsafe { miri_write_to_stdout(_buf) };
        }
        Ok(())
    }

    // Keep the default write_fmt so the `fmt::Arguments` are still evaluated.

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Stderr {
    pub const fn new() -> Stderr {
        Stderr
    }
}

impl io::Write for Stderr {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        #[cfg(verus_explorer)]
        unsafe { miri_write_to_stderr(buf) };
        Ok(buf.len())
    }

    #[inline]
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        let mut total_len = 0;
        for buf in bufs {
            #[cfg(verus_explorer)]
            unsafe { miri_write_to_stderr(buf) };
            total_len += buf.len();
        }
        Ok(total_len)
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn write_all(&mut self, _buf: &[u8]) -> io::Result<()> {
        #[cfg(verus_explorer)]
        unsafe { miri_write_to_stderr(_buf) };
        Ok(())
    }

    #[inline]
    fn write_all_vectored(&mut self, _bufs: &mut [IoSlice<'_>]) -> io::Result<()> {
        for _buf in _bufs.iter() {
            #[cfg(verus_explorer)]
            unsafe { miri_write_to_stderr(_buf) };
        }
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub const STDIN_BUF_SIZE: usize = 0;

pub fn is_ebadf(_err: &io::Error) -> bool {
    true
}

pub fn panic_output() -> Option<Vec<u8>> {
    None
}

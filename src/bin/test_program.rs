use std::{
    os::fd::{AsFd, FromRawFd},
    time::SystemTime,
};

use rustix::io::retry_on_intr;

fn main() {
    let confirm = unsafe { rustix::fd::OwnedFd::from_raw_fd(12345) };
    write_all(
        confirm,
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros()
            .to_string()
            .as_bytes(),
    )
    .unwrap();
}

fn write_all(fd: impl AsFd, buf: &[u8]) -> rustix::io::Result<()> {
    let mut written = 0;
    loop {
        let n = retry_on_intr(|| rustix::io::write(&fd, &buf[written..]))?;
        if n == 0 {
            return Err(rustix::io::Errno::CANCELED);
        }

        written += n;
        if written == buf.len() {
            return Ok(());
        }
    }
}

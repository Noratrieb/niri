/*
[dependencies]
anyhow = "1.0.102"
libc = "0.2.186"
rustix = { version = "1.1.4", features = ["pipe"] }
zbus = "5.16.0"
*/
use std::{
    io::Write,
    os::fd::{FromRawFd, OwnedFd},
    process::Command,
    time::SystemTime,
};

use rustix::pipe::PipeFlags;

mod current;
mod synced;

fn main() {
    measure("current", |cmd| {
        current::do_spawn(&cmd.get_program().to_os_string(), cmd).unwrap()
    });
    measure("synced", |cmd| {
        synced::do_spawn(&cmd.get_program().to_os_string(), cmd).unwrap()
    });
}

fn measure<R>(scenario: &str, f: impl Fn(Command) -> R) {
    let iter = 200;

    let mut durations = vec![];

    std::io::stdout().lock().flush().unwrap();

    for i in 0..iter {
        let (pipe_read, pipe_write) = rustix::pipe::pipe_with(PipeFlags::empty()).unwrap();
        let mut read_buf = [0; 1024];

        let mut pipe_write_child_fd = unsafe { OwnedFd::from_raw_fd(12345) };
        rustix::io::dup2(pipe_write, &mut pipe_write_child_fd).unwrap();

        let cmd = Command::new("./target/release/test_program");

        let start_us = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros();

        f(cmd);

        let len = rustix::io::read(pipe_read, &mut read_buf).unwrap();
        let result_str = str::from_utf8(&read_buf[..len]).unwrap();
        let end_us = result_str.parse::<u128>().unwrap();

        let duration_us = end_us - start_us;

        durations.push((duration_us as f32) / 1000.);

        print!("\r{i}/{iter}");
        std::io::stdout().lock().flush().unwrap();
    }

    let min = durations.iter().copied().min_by(f32::total_cmp).unwrap();
    let max = durations.iter().copied().max_by(f32::total_cmp).unwrap();
    let avg = durations.iter().sum::<f32>() / durations.len() as f32;

    println!("\r{scenario}: iter {iter}, min {min}ms, max {max}ms, avg {avg}ms");
}

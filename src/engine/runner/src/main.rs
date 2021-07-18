use std::{io::{Read, stdin}, time::{self, Instant}};

use platform::{timer::Timer, window};

fn main() {
    let mut prev = Instant::now();

    let _timer = Timer::new(0, 1000, &mut || {
        let now = Instant::now();
        let since = now.duration_since(prev);
        println!("{:?}: {:?}", since, window::Window::foreground());
        prev = now;
    });
    let mut s = String::new();
    stdin().read_to_string(&mut s);
}
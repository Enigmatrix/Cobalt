#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]

pub mod error;
pub mod buffer;
pub mod process;
pub mod window;
pub mod timer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

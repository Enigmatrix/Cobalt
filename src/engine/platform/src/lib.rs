#![feature(new_uninit)]
#![feature(maybe_uninit_slice)]

pub mod error;
pub mod buffer;
pub mod timer;
pub mod window;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

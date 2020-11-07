// extern crate engine;

#[test]
fn it_works() {
    let err = native::error::Win32Err::from(6);
    assert_eq!(
        err.to_string(),
        "Win32(0x6): The handle is invalid. (os error 6)"
    )
}

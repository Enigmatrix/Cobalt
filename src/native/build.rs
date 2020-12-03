fn main() {
    winrt::build!(
        windows::applicationmodel::*
        windows::storage::streams::*
        windows::security::cryptography::{CryptographicBuffer}
    );
}

use std::cell::RefCell;

use native::watchers::*;
use std::rc::Rc;

mod common;

use common::*;

#[test]
fn switch_window() {
    native::setup().unwrap();
    pause(); // give some time for the first GetMessage() to be called

    let switches = Rc::new(RefCell::new(Vec::new()));
    let notepad = "C:\\Windows\\system32\\notepad.exe";
    let mail = "microsoft.windowscommunicationsapps_8wekyb3d8bbwe!microsoft.windowslive.mail";

    let mail = uwp::App::spawn(mail);
    let notepad = win32::App::spawn(notepad);

    let _fgw = {
        let switches = Rc::clone(&switches);
        foreground::Watcher::new(|window, _| {
            switches.borrow_mut().push(window);
            Ok(())
        })
        .unwrap()
    };

    bring_to_foreground(&mail);
    assert_eq!(switches.borrow().last(), main_window(&mail).as_ref());

    bring_to_foreground(&notepad);
    assert_eq!(switches.borrow().last(), main_window(&notepad).as_ref());

    bring_to_foreground(&mail);
    assert_eq!(switches.borrow().last(), main_window(&mail).as_ref());

    assert_eq!(switches.borrow().len(), 3);
}

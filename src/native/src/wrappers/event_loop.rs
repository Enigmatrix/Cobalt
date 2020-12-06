use crate::raw::*;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::task::{Context as FutContext, Poll};

pub struct EventLoop {
    msg: winuser::MSG,
}

impl Default for EventLoop {
    fn default() -> Self {
        EventLoop::new()
    }
}

impl EventLoop {
    pub fn new() -> EventLoop {
        EventLoop {
            msg: winuser::MSG::default(),
        }
    }
}

impl EventLoop {
    pub fn step_peek(&mut self) -> Option<usize> {
        while unsafe {
            winuser::PeekMessageW(&mut self.msg, ptr::null_mut(), 0, 0, winuser::PM_REMOVE)
        } != 0
        {
            if self.msg.message == winuser::WM_QUIT {
                return Some(self.msg.wParam);
            }
            unsafe { winuser::TranslateMessage(&mut self.msg as *mut _) };
            unsafe { winuser::DispatchMessageW(&mut self.msg as *mut _) };
        }
        None
    }

    pub fn step(&mut self) -> Option<usize> {
        if unsafe { winuser::GetMessageW(&mut self.msg, ptr::null_mut(), 0, 0) } != 0 {
            if self.msg.message == winuser::WM_QUIT {
                return Some(self.msg.wParam);
            }
            unsafe { winuser::TranslateMessage(&mut self.msg as *mut _) };
            unsafe { winuser::DispatchMessageW(&mut self.msg as *mut _) };
        }
        None
    }

    pub fn run(&mut self) -> Option<usize> {
        loop {
            if let Some(ex) = self.step() {
                return Some(ex);
            }
        }
    }
}

impl Future for EventLoop {
    type Output = usize;

    fn poll(mut self: Pin<&mut Self>, cx: &mut FutContext<'_>) -> Poll<usize> {
        if let Some(exit) = self.step_peek() {
            Poll::Ready(exit)
        } else {
            cx.waker().wake_by_ref(); // yield to scheduler
            Poll::Pending
        }
    }
}

use std::ffi::c_void;
use std::mem::ManuallyDrop;

use handler::*;
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationCacheRequest, IUIAutomationElement9,
    IUIAutomationPropertyChangedEventHandler, TreeScope, UIA_PROPERTY_ID,
};
use windows::core::{AgileReference, Interface, implement, interface};

/// Callback for property change event
pub type Callback = Box<dyn Fn(UIA_PROPERTY_ID, String) -> Result<()>>;

/// Property changed event handler
pub struct PropertyChange {
    automation: AgileReference<IUIAutomation>,
    element: AgileReference<IUIAutomationElement9>,
    handler_ref: AgileReference<IUIAutomationPropertyChangedEventHandler>,
}

impl PropertyChange {
    /// Create a new property changed event handler for the element.
    pub fn new(
        automation: AgileReference<IUIAutomation>,
        element: AgileReference<IUIAutomationElement9>,
        scope: TreeScope,
        cache_request: Option<IUIAutomationCacheRequest>,
        properties: &[UIA_PROPERTY_ID],
        callback: Callback,
    ) -> Result<Self> {
        let handler = IPropertyChangedEventHandler::from(Handler::new(callback));
        let handler_ref: IUIAutomationPropertyChangedEventHandler = handler.cast()?;
        let handler_ref = AgileReference::new(&handler_ref)?;

        unsafe {
            automation
                .resolve()?
                .AddPropertyChangedEventHandlerNativeArray(
                    &element.resolve()?,
                    scope,
                    cache_request.as_ref(),
                    &handler_ref.resolve()?,
                    properties,
                )?
        }

        Ok(Self {
            automation,
            element,
            handler_ref,
        })
    }
}

impl Drop for PropertyChange {
    fn drop(&mut self) {
        // During process shutdown (LdrShutdownProcess), COM TLS is already torn
        // down and AgileReference::resolve() will fail. Bail silently instead of
        // panicking — the OS reclaims everything on exit anyway.
        let Ok(automation) = self.automation.resolve() else {
            return;
        };
        let Ok(element) = self.element.resolve() else {
            return;
        };
        let Ok(handler) = self.handler_ref.resolve() else {
            return;
        };
        unsafe {
            automation
                .RemovePropertyChangedEventHandler(&element, &handler)
                .context("drop property change handler")
                .error();
        }
    }
}

#[allow(non_snake_case)]
mod handler {
    use util::error::Context;
    use util::tracing::ResultTraceExt;
    use windows::Win32::Foundation::S_OK;
    use windows::core::{HRESULT, IUnknown, IUnknown_Vtbl};

    use super::*;

    /// ABI-compatible redeclaration of [IUIAutomationPropertyChangedEventHandler] (same IID).
    ///
    /// UIA passes `newValue` by value but keeps ownership and clears it after the callback
    /// returns. The windows-rs generated shim drops the by-value [VARIANT] (`VariantClear`),
    /// so UIA's own clear becomes a double free that corrupts the heap
    /// (<https://github.com/microsoft/windows-rs/issues/3818>). [ManuallyDrop] has the same
    /// ABI as [VARIANT] but never clears it.
    #[interface("40cd37d4-c756-4b0c-8c6f-bddfeeb13b50")]
    pub unsafe trait IPropertyChangedEventHandler: IUnknown {
        fn HandlePropertyChangedEvent(
            &self,
            sender: *mut c_void,
            property_id: UIA_PROPERTY_ID,
            new_value: ManuallyDrop<VARIANT>,
        ) -> HRESULT;
    }

    #[allow(missing_docs)]
    #[implement(IPropertyChangedEventHandler)]
    /// Property changed event handler
    pub struct Handler {
        inner: Callback,
    }

    impl Handler {
        /// Create a new property value changed event handler
        pub fn new(inner: Callback) -> Self {
            Self { inner }
        }
    }

    impl IPropertyChangedEventHandler_Impl for Handler_Impl {
        unsafe fn HandlePropertyChangedEvent(
            &self,
            _sender: *mut c_void,
            property_id: UIA_PROPERTY_ID,
            new_value: ManuallyDrop<VARIANT>,
        ) -> HRESULT {
            let value = new_value.to_string();
            (self.inner)(property_id, value)
                .context("prop value changed")
                .error();
            S_OK
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance};
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, TreeScope_Element, UIA_ValueValuePropertyId,
    };
    use windows::core::IUnknown;

    use super::*;

    fn new_handler(callback: Callback) -> IUIAutomationPropertyChangedEventHandler {
        IPropertyChangedEventHandler::from(Handler::new(callback))
            .cast()
            .expect("cast to IUIAutomationPropertyChangedEventHandler")
    }

    fn ref_count(unknown: &IUnknown) -> u32 {
        unsafe {
            let vtable = unknown.vtable();
            (vtable.AddRef)(unknown.as_raw());
            (vtable.Release)(unknown.as_raw())
        }
    }

    #[test]
    fn handler_receives_value() -> Result<()> {
        let received = Arc::new(Mutex::new(Vec::new()));
        let handler = new_handler({
            let received = received.clone();
            Box::new(move |_, value| {
                received.lock().expect("received lock").push(value);
                Ok(())
            })
        });

        let expected = vec!["https://example.com/".to_string(), "".to_string()];
        for value in &expected {
            let new_value = VARIANT::from(value.as_str());
            unsafe {
                handler.HandlePropertyChangedEvent(None, UIA_ValueValuePropertyId, &new_value)?
            };
        }

        assert_eq!(*received.lock().expect("received lock"), expected);
        Ok(())
    }

    /// Invokes the handler through its COM vtable the same way UIA does: the caller owns
    /// `newValue` and clears it after the call returns, so the handler must not clear it too.
    #[test]
    fn handler_does_not_release_caller_owned_variant() -> Result<()> {
        let handler = new_handler(Box::new(|_, _| Ok(())));
        let payload: IUnknown = new_handler(Box::new(|_, _| Ok(()))).cast()?;
        // Leak one reference so a handler that wrongly releases the payload fails the assert
        // below instead of freeing the object out from under the test.
        std::mem::forget(payload.clone());

        let baseline = ref_count(&payload);
        let new_value = VARIANT::from(payload.clone());
        let before = ref_count(&payload);
        unsafe { handler.HandlePropertyChangedEvent(None, UIA_ValueValuePropertyId, &new_value)? };
        assert_eq!(ref_count(&payload), before);

        drop(new_value);
        assert_eq!(ref_count(&payload), baseline);
        Ok(())
    }

    #[test]
    fn property_change_releases_handler_when_dropped() -> Result<()> {
        crate::setup_thread_com()?;
        let automation: IUIAutomation =
            unsafe { CoCreateInstance(&CUIAutomation, None, CLSCTX_ALL)? };
        let element: IUIAutomationElement9 = unsafe { automation.GetRootElement()? }.cast()?;

        let captured = Arc::new(());
        let callback: Callback = {
            let captured = captured.clone();
            Box::new(move |_, _| {
                let _ = &captured;
                Ok(())
            })
        };
        let property_change = PropertyChange::new(
            AgileReference::new(&automation)?,
            AgileReference::new(&element)?,
            TreeScope_Element,
            None,
            &[UIA_ValueValuePropertyId],
            callback,
        )?;
        assert_eq!(Arc::strong_count(&captured), 2);

        drop(property_change);
        assert_eq!(Arc::strong_count(&captured), 1);
        Ok(())
    }
}

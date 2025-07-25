use handler::*;
use util::error::{Context, Result};
use util::tracing::ResultTraceExt;
use windows::Win32::System::Variant::VARIANT;
use windows::Win32::UI::Accessibility::{
    IUIAutomation, IUIAutomationCacheRequest, IUIAutomationElement, IUIAutomationElement9,
    IUIAutomationPropertyChangedEventHandler, IUIAutomationPropertyChangedEventHandler_Impl,
    TreeScope, UIA_PROPERTY_ID,
};
use windows::core::{AgileReference, implement};

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
        let handler = Handler::new(callback);
        let handler_ref = IUIAutomationPropertyChangedEventHandler::from(handler);
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
        unsafe {
            self.automation
                .resolve()
                .expect("automation")
                .RemovePropertyChangedEventHandler(
                    &self.element.resolve().expect("element"),
                    &self.handler_ref.resolve().expect("handler"),
                )
                .context("drop property change handler")
                .error();
        }
    }
}

mod handler {
    use util::error::Context;
    use util::tracing::ResultTraceExt;

    use super::*;

    #[allow(missing_docs)]
    #[implement(IUIAutomationPropertyChangedEventHandler)]
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

    impl IUIAutomationPropertyChangedEventHandler_Impl for Handler_Impl {
        #[allow(non_snake_case)]
        fn HandlePropertyChangedEvent(
            &self,
            _sender: windows_core::Ref<'_, IUIAutomationElement>,
            eventid: UIA_PROPERTY_ID,
            value: &VARIANT,
        ) -> windows::core::Result<()> {
            let value = value.to_string();
            (self.inner)(eventid, value)
                .context("prop value changed")
                .error();
            Ok(())
        }
    }
}

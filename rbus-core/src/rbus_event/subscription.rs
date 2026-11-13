use super::RBusEventRef;
use crate::{RBusError, RBusHandle, RBusHandleRef, RBusLibrary};
use rbus_sys::{rbusEvent_t, rbusEventSubscription_t, rbusHandle_t};
use std::any::Any;
use std::ffi::{CStr, CString};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::time::Duration;

///
/// Operations for subscribing to RBus events.
///
pub struct RBusEvent;

/// Options for establishing a subscription.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RBusSubscribeOptions {
    /// Time to retry when no provider is available, rounded up to whole seconds.
    /// Zero disables retries. This does not limit the subscription's lifetime.
    pub timeout: Duration,
}

///
/// A subscription borrowing its owning connection.
/// Keep this guard alive to receive events; dropping it unsubscribes.
/// Use drop(subscription) to stop listening before the end of its scope.
///
/// ```compile_fail,E0597
/// use rbus_core::{RBusEvent, RBusHandle};
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let subscription = {
///         let bus = RBusHandle::open(c"subscriber")?;
///         RBusEvent::subscribe(&bus, c"Device.Test.Event!", |_| {})?
///     };
///     drop(subscription);
///     Ok(())
/// }
/// ```
#[must_use = "keep the subscription alive to receive events"]
pub struct RBusEventSubscription<'a> {
    handle: &'a RBusHandle,
    event_name: CString,
    // Keep the handler alive until native unsubscription has finished.
    _event_handler: Box<dyn Any + Send>,
}

unsafe extern "C" fn dispatch_event<F>(
    handle: rbusHandle_t,
    event_data: *const rbusEvent_t,
    subscription: *mut rbusEventSubscription_t,
) where
    F: for<'event> FnMut(RBusEventRef<'event>) + Send + 'static,
{
    // Catch Rust panics before returning to C.
    let _ = catch_unwind(AssertUnwindSafe(|| {
        if handle.is_null() {
            return;
        }
        // SAFETY: RBus owns the event and subscription for this callback.
        let (Some(event_data), Some(subscription)) = (unsafe { event_data.as_ref() }, unsafe {
            subscription.as_ref()
        }) else {
            return;
        };
        if event_data.name.is_null() {
            return;
        }
        // SAFETY: userData points to the boxed F owned by the guard.
        // Native RBus serializes callbacks for this connection.
        let Some(handler) = (unsafe { subscription.userData.cast::<F>().as_mut() }) else {
            return;
        };
        // SAFETY: RBus keeps the connection valid until this callback returns.
        let handle = unsafe { RBusHandleRef::from_raw(&handle) };
        handler(RBusEventRef {
            handle,
            event: event_data,
        });
    }));
}

impl RBusEvent {
    ///
    /// Subscribe with no retry when no provider is available.
    /// The callback runs on an RBus thread and may begin before this returns.
    /// Callback panics are caught before returning to C.
    /// Create and drop subscriptions from outside event callbacks.
    ///
    pub fn subscribe<'a, F>(
        handle: &'a RBusHandle,
        event_name: &CStr,
        handler: F,
    ) -> Result<RBusEventSubscription<'a>, RBusError>
    where
        F: for<'event> FnMut(RBusEventRef<'event>) + Send + 'static,
    {
        Self::subscribe_with_options(handle, event_name, RBusSubscribeOptions::default(), handler)
    }

    ///
    /// Subscribe with options controlling retries during establishment.
    ///
    pub fn subscribe_with_options<'a, F>(
        handle: &'a RBusHandle,
        event_name: &CStr,
        options: RBusSubscribeOptions,
        handler: F,
    ) -> Result<RBusEventSubscription<'a>, RBusError>
    where
        F: for<'event> FnMut(RBusEventRef<'event>) + Send + 'static,
    {
        let mut timeout =
            i32::try_from(options.timeout.as_secs()).map_err(|_| RBusError::InvalidInput)?;

        // Round fractional seconds up to the next whole second.
        if options.timeout.subsec_nanos() != 0 {
            timeout = timeout.checked_add(1).ok_or(RBusError::InvalidInput)?;
        }

        // Native RBus multiplies the timeout by 1000 using a signed int.
        if timeout > i32::MAX / 1000 {
            return Err(RBusError::InvalidInput);
        }

        let event_name = event_name.to_owned();
        // The Box keeps userData stable when the subscription moves.
        let mut handler = Box::new(handler);
        let result = unsafe {
            RBusLibrary::get().rbusEvent_Subscribe(
                handle.handle,
                event_name.as_ptr(),
                Some(dispatch_event::<F>),
                std::ptr::from_mut(handler.as_mut()).cast(),
                timeout,
            )
        };
        RBusError::map(result, || RBusEventSubscription {
            handle,
            event_name,
            _event_handler: handler,
        })
    }
}

impl Drop for RBusEventSubscription<'_> {
    fn drop(&mut self) {
        unsafe {
            RBusLibrary::get().rbusEvent_Unsubscribe(self.handle.handle, self.event_name.as_ptr())
        };
    }
}

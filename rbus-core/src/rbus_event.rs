mod subscription;

pub use subscription::{RBusEvent, RBusEventSubscription, RBusSubscribeOptions};

use crate::{RBusHandleRef, RBusObjectRef};
use rbus_sys::{rbusEvent_t, rbusEventType_t};
use std::ffi::CStr;
use std::os::raw::c_uint;

/// The type of a received RBus event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RBusEventType {
    ObjectCreated,
    ObjectDeleted,
    ValueChanged,
    General,
    InitialValue,
    Interval,
    DurationComplete,
    Unexpected(c_uint),
}

impl RBusEventType {
    fn from_raw(value: rbusEventType_t) -> Self {
        match value {
            rbusEventType_t::RBUS_EVENT_OBJECT_CREATED => Self::ObjectCreated,
            rbusEventType_t::RBUS_EVENT_OBJECT_DELETED => Self::ObjectDeleted,
            rbusEventType_t::RBUS_EVENT_VALUE_CHANGED => Self::ValueChanged,
            rbusEventType_t::RBUS_EVENT_GENERAL => Self::General,
            rbusEventType_t::RBUS_EVENT_INITIAL_VALUE => Self::InitialValue,
            rbusEventType_t::RBUS_EVENT_INTERVAL => Self::Interval,
            rbusEventType_t::RBUS_EVENT_DURATION_COMPLETE => Self::DurationComplete,
            other => Self::Unexpected(other.0),
        }
    }
}

///
/// An event borrowed for the duration of its callback.
/// The event, connection and data cannot be stored beyond that callback.
///
/// ```compile_fail
/// use rbus_core::RBusEventRef;
/// fn keep(event: RBusEventRef<'_>) -> RBusEventRef<'static> { event }
/// ```
#[derive(Clone, Copy)]
pub struct RBusEventRef<'a> {
    handle: RBusHandleRef<'a>,
    event: &'a rbusEvent_t,
}

impl<'a> RBusEventRef<'a> {
    ///
    /// Borrow the connection receiving this event.
    ///
    /// ```compile_fail
    /// use rbus_core::{RBusEventRef, RBusHandleRef};
    /// fn keep(event: RBusEventRef<'_>) -> RBusHandleRef<'static> { event.handle() }
    /// ```
    pub fn handle(&self) -> RBusHandleRef<'a> {
        self.handle
    }

    /// Get the fully qualified event name.
    pub fn name(&self) -> &'a CStr {
        // SAFETY: the callback validates the name and limits the event's lifetime.
        unsafe { CStr::from_ptr(self.event.name) }
    }

    /// Get the event type.
    pub fn kind(&self) -> RBusEventType {
        RBusEventType::from_raw(self.event.type_)
    }

    /// Borrow the event payload, if present, without copying it.
    pub fn data(&self) -> Option<RBusObjectRef<'a>> {
        if self.event.data.is_null() {
            None
        } else {
            // SAFETY: RBus keeps event data valid until its callback returns.
            Some(unsafe { RBusObjectRef::from_raw(&self.event.data) })
        }
    }
}

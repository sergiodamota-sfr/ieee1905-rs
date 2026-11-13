use crate::{RBusLibrary, RBusProperties, RBusProperty};
use rbus_sys::rbusObject_t;
use std::ffi::{CStr, CString};

///
/// An object borrowed from an owner or an event callback.
/// Dropping the view does not release the native object.
///
#[derive(Clone, Copy)]
pub struct RBusObjectRef<'a> {
    pub(super) handle: &'a rbusObject_t,
}

impl<'a> RBusObjectRef<'a> {
    ///
    /// # Safety
    /// The object must be non-null and valid for the borrowed lifetime.
    /// The shared library must already be initialized.
    ///
    pub(crate) unsafe fn from_raw(handle: &'a rbusObject_t) -> Self {
        Self { handle }
    }

    /// Get the name of the object.
    pub fn get_name(&self) -> CString {
        let name = unsafe { RBusLibrary::get().rbusObject_GetName(*self.handle) };
        if name.is_null() {
            CString::default()
        } else {
            unsafe { CStr::from_ptr(name) }.to_owned()
        }
    }

    /// Get the retained property list, which may outlive this view.
    pub fn get_properties(&self) -> RBusProperties {
        let properties = unsafe { RBusLibrary::get().rbusObject_GetProperties(*self.handle) };
        RBusProperties::retain(properties)
    }

    /// Get a retained property, which may outlive this view.
    pub fn get_property(&self, name: &CStr) -> Option<RBusProperty> {
        let property =
            unsafe { RBusLibrary::get().rbusObject_GetProperty(*self.handle, name.as_ptr()) };
        if property.is_null() {
            None
        } else {
            Some(RBusProperty::retain(property))
        }
    }
}

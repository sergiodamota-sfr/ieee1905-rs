mod reference;

use crate::rbus_library::{RBusLibrary, RBusOpenError};
use crate::rbus_value::RBusValue;
use crate::{RBusObject, RBusValueGetError, RBusValueReadable, RBusValueWritable};
use rbus_sys::*;
pub use reference::RBusPropertyRef;
use std::cmp::Ordering;
use std::ffi::{CStr, CString};

pub struct RBusProperty {
    pub(super) handle: rbusProperty_t,
}

impl RBusProperty {
    ///
    /// Allocate and initialize a property.
    /// Loads the shared library if needed and returns an error if it is unavailable.
    ///
    pub fn new(name: &CStr, value: &RBusValue) -> Result<Self, RBusOpenError> {
        RBusLibrary::load()?;
        let library_raw = RBusLibrary::get();
        let handle = unsafe {
            library_raw.rbusProperty_Init(std::ptr::null_mut(), name.as_ptr(), value.handle)
        };
        Ok(Self { handle })
    }

    ///
    /// Borrow this property without retaining its native handle.
    ///
    pub fn as_ref(&self) -> RBusPropertyRef<'_> {
        RBusPropertyRef {
            handle: &self.handle,
        }
    }

    pub(crate) fn retain(handle: rbusProperty_t) -> Self {
        unsafe {
            RBusLibrary::get().rbusProperty_Retain(handle);
        }
        Self { handle }
    }

    ///
    /// Get the name of a property.
    ///
    pub fn get_name(&self) -> CString {
        self.as_ref().get_name()
    }

    ///
    /// Set the name of a property.
    ///
    pub fn set_name(&self, name: &CStr) {
        self.as_ref().set_name(name)
    }

    ///
    /// Get the value of a property.
    ///
    pub fn get<T>(&self) -> Result<T, RBusValueGetError>
    where
        T: RBusValueReadable,
    {
        self.as_ref().get()
    }

    ///
    /// Get the value of a property.
    ///
    pub fn get_value(&self) -> RBusValue {
        self.as_ref().get_value()
    }

    ///
    /// Set the value of a property.
    ///
    pub fn set<T>(&self, value: &T)
    where
        T: RBusValueWritable + ?Sized,
    {
        self.as_ref().set(value)
    }

    ///
    /// Set the value of a property.
    ///
    pub fn set_value(&self, value: &RBusValue) {
        self.as_ref().set_value(value)
    }

    ///
    /// Set the object of a property.
    ///
    pub fn set_object(&self, value: &RBusObject) {
        self.as_ref().set_object(value)
    }

    ///
    /// Set the property of a property.
    ///
    pub fn set_property(&self, value: &RBusProperty) {
        self.as_ref().set_property(value)
    }

    ///
    /// Append a property to the end of a property list.  
    ///
    pub fn append_property(&self, value: &RBusProperty) {
        self.as_ref().append_property(value)
    }
}

impl Drop for RBusProperty {
    fn drop(&mut self) {
        let library = RBusLibrary::get();
        unsafe {
            library.rbusProperty_Release(self.handle);
        }
    }
}

impl Clone for RBusProperty {
    fn clone(&self) -> Self {
        let library = RBusLibrary::get();
        unsafe {
            library.rbusProperty_Retain(self.handle);
        }
        Self {
            handle: self.handle,
        }
    }
}

impl Eq for RBusProperty {}

impl PartialEq for RBusProperty {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Ord for RBusProperty {
    fn cmp(&self, other: &Self) -> Ordering {
        let library = RBusLibrary::get();
        let result = unsafe { library.rbusProperty_Compare(self.handle, other.handle) };
        if result < 0 {
            return Ordering::Less;
        }
        if result > 0 {
            return Ordering::Greater;
        }
        Ordering::Equal
    }
}

impl PartialOrd for RBusProperty {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

use super::RBusProperty;
use crate::{
    RBusLibrary, RBusObject, RBusValue, RBusValueGetError, RBusValueReadable, RBusValueWritable,
};
use rbus_sys::rbusProperty_t;
use std::ffi::{CStr, CString};

///
/// A property borrowed from an owning wrapper or a native callback.
/// Dropping this view does not release the native property.
///
/// ```compile_fail,E0597
/// use rbus_core::{RBusLibrary, RBusProperty, RBusValue};
/// fn example(library: &RBusLibrary) {
///     let borrowed = {
///         let value = RBusValue::from(library, &42_i32);
///         let owner = RBusProperty::new(library, c"borrowed-property", &value);
///         owner.as_ref()
///     };
///     borrowed.get_name();
/// }
/// ```
#[derive(Clone, Copy)]
pub struct RBusPropertyRef<'a> {
    pub(super) handle: rbusProperty_t,
    pub(super) library: &'a RBusLibrary,
}

impl<'a> RBusPropertyRef<'a> {
    ///
    /// Borrow a native property without retaining it.
    ///
    /// # Safety
    /// The property must remain valid for the entire borrowed lifetime.
    ///
    pub(crate) unsafe fn from_raw(library: &'a RBusLibrary, handle: rbusProperty_t) -> Self {
        Self { handle, library }
    }

    /// See [`RBusProperty::get_name`].
    pub fn get_name(&self) -> CString {
        let library = self.library.as_raw();
        unsafe {
            let ptr = library.rbusProperty_GetName(self.handle);
            if ptr.is_null() {
                return CString::default();
            }
            CStr::from_ptr(ptr).to_owned()
        }
    }

    /// See [`RBusProperty::set_name`].
    pub fn set_name(&self, name: &CStr) {
        let library = self.library.as_raw();
        unsafe {
            library.rbusProperty_SetName(self.handle, name.as_ptr());
        }
    }

    /// See [`RBusProperty::get`].
    pub fn get<T>(&self) -> Result<T, RBusValueGetError>
    where
        T: RBusValueReadable,
    {
        self.get_value().get()
    }

    /// See [`RBusProperty::get_value`].
    pub fn get_value(&self) -> RBusValue {
        let library = self.library.as_raw();
        unsafe {
            let handle = library.rbusProperty_GetValue(self.handle);
            RBusValue::retain(self.library, handle)
        }
    }

    /// See [`RBusProperty::set`].
    pub fn set<T>(&self, value: &T)
    where
        T: RBusValueWritable + ?Sized,
    {
        self.set_value(&RBusValue::from(self.library, value));
    }

    /// See [`RBusProperty::set_value`].
    pub fn set_value(&self, value: &RBusValue) {
        let library = self.library.as_raw();
        unsafe {
            library.rbusProperty_SetValue(self.handle, value.handle);
        }
    }

    /// See [`RBusProperty::set_object`].
    pub fn set_object(&self, value: &RBusObject) {
        let library = self.library.as_raw();
        unsafe {
            library.rbusProperty_SetObject(self.handle, value.handle);
        }
    }

    /// See [`RBusProperty::set_property`].
    pub fn set_property(&self, value: &RBusProperty) {
        let library = self.library.as_raw();
        unsafe {
            library.rbusProperty_SetProperty(self.handle, value.handle);
        }
    }

    /// See [`RBusProperty::append_property`].
    pub fn append_property(&self, value: &RBusProperty) {
        let library = self.library.as_raw();
        unsafe {
            library.rbusProperty_Append(self.handle, value.handle);
        }
    }
}

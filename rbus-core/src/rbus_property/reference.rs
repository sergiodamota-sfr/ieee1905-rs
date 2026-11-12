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
/// use rbus_core::{RBusProperty, RBusValue};
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let borrowed = {
///         let value = RBusValue::from(&42_i32)?;
///         let owner = RBusProperty::new(c"borrowed-property", &value)?;
///         owner.as_ref()
///     };
///     borrowed.get_name();
///     Ok(())
/// }
/// ```
#[derive(Clone, Copy)]
pub struct RBusPropertyRef<'a> {
    pub(super) handle: &'a rbusProperty_t,
}

impl<'a> RBusPropertyRef<'a> {
    ///
    /// Borrow a native property without retaining it.
    ///
    /// # Safety
    /// The property must remain valid for the entire borrowed lifetime.
    /// The shared library must already be initialized.
    ///
    pub(crate) unsafe fn from_raw(handle: &'a rbusProperty_t) -> Self {
        Self { handle }
    }

    /// See [`RBusProperty::get_name`].
    pub fn get_name(&self) -> CString {
        let library = RBusLibrary::get();
        unsafe {
            let ptr = library.rbusProperty_GetName(*self.handle);
            if ptr.is_null() {
                return CString::default();
            }
            CStr::from_ptr(ptr).to_owned()
        }
    }

    /// See [`RBusProperty::set_name`].
    pub fn set_name(&self, name: &CStr) {
        let library = RBusLibrary::get();
        unsafe {
            library.rbusProperty_SetName(*self.handle, name.as_ptr());
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
        let library = RBusLibrary::get();
        unsafe {
            let handle = library.rbusProperty_GetValue(*self.handle);
            RBusValue::retain(handle)
        }
    }

    /// See [`RBusProperty::set`].
    pub fn set<T>(&self, value: &T)
    where
        T: RBusValueWritable + ?Sized,
    {
        self.set_value(&RBusValue::from_initialized(value));
    }

    /// See [`RBusProperty::set_value`].
    pub fn set_value(&self, value: &RBusValue) {
        let library = RBusLibrary::get();
        unsafe {
            library.rbusProperty_SetValue(*self.handle, value.handle);
        }
    }

    /// See [`RBusProperty::set_object`].
    pub fn set_object(&self, value: &RBusObject) {
        let library = RBusLibrary::get();
        unsafe {
            library.rbusProperty_SetObject(*self.handle, value.handle);
        }
    }

    /// See [`RBusProperty::set_property`].
    pub fn set_property(&self, value: &RBusProperty) {
        let library = RBusLibrary::get();
        unsafe {
            library.rbusProperty_SetProperty(*self.handle, value.handle);
        }
    }

    /// See [`RBusProperty::append_property`].
    pub fn append_property(&self, value: &RBusProperty) {
        let library = RBusLibrary::get();
        unsafe {
            library.rbusProperty_Append(*self.handle, value.handle);
        }
    }
}

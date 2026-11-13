use super::{RBusDataElement, RBusError, RBusGetError, RBusRowNames, RBusTableSyncHandler};
use crate::{RBusLibrary, RBusValue, RBusValueReadable, RBusValueWritable};
use rbus_sys::{rbusError_t, rbusHandle_t, rbusValue_t};
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

///
/// A connection borrowed from an owning handle or a native callback.
/// Dropping this view does not close the connection.
///
/// ```compile_fail,E0597
/// use rbus_core::RBusHandle;
/// fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let borrowed = {
///         let owner = RBusHandle::open(c"borrowed-connection")?;
///         owner.as_ref()
///     };
///     borrowed.get::<i32>(c"Device.Test.Value")?;
///     Ok(())
/// }
/// ```
#[derive(Clone, Copy)]
pub struct RBusHandleRef<'a> {
    pub(super) handle: &'a rbusHandle_t,
}

impl<'a> RBusHandleRef<'a> {
    ///
    /// Borrow a native connection without taking ownership.
    ///
    /// # Safety
    /// The connection must remain valid for the entire borrowed lifetime.
    /// The shared library must already be initialized.
    ///
    pub(crate) unsafe fn from_raw(handle: &'a rbusHandle_t) -> Self {
        Self { handle }
    }

    /// See [`crate::RBusHandle::register_data_elements`].
    pub fn register_data_elements(&self, elements: &[RBusDataElement]) -> Result<(), RBusError> {
        let library = RBusLibrary::get();
        let result = unsafe {
            library.rbus_regDataElements(
                *self.handle,
                elements.len() as c_int,
                elements.as_ptr().cast_mut().cast(),
            )
        };
        RBusError::map(result, || ())
    }

    /// See [`crate::RBusHandle::unregister_data_elements`].
    pub fn unregister_data_elements(&self, elements: &[RBusDataElement]) -> Result<(), RBusError> {
        let library = RBusLibrary::get();
        let result = unsafe {
            library.rbus_unregDataElements(
                *self.handle,
                elements.len() as c_int,
                elements.as_ptr().cast_mut().cast(),
            )
        };
        RBusError::map(result, || ())
    }

    /// See [`crate::RBusHandle::register_table_row`].
    pub fn register_table_row(
        &self,
        table_name: &CStr,
        instance_id: u32,
        alias_name: Option<&CStr>,
    ) -> Result<(), RBusError> {
        let library = RBusLibrary::get();
        let result = unsafe {
            library.rbusTable_registerRow(
                *self.handle,
                table_name.as_ptr(),
                instance_id,
                alias_name.map(|e| e.as_ptr()).unwrap_or_default(),
            )
        };
        RBusError::map(result, || ())
    }

    /// See [`crate::RBusHandle::unregister_table_row`].
    pub fn unregister_table_row(&self, row_name: &CStr) -> Result<(), RBusError> {
        let library = RBusLibrary::get();
        let result = unsafe { library.rbusTable_unregisterRow(*self.handle, row_name.as_ptr()) };
        RBusError::map(result, || ())
    }

    /// See [`crate::RBusHandle::register_dynamic_table_sync_handler`].
    pub fn register_dynamic_table_sync_handler<T>(&self, table_name: &CStr) -> Result<(), RBusError>
    where
        T: RBusTableSyncHandler,
    {
        unsafe extern "C" fn handler<T: RBusTableSyncHandler>(
            handle: rbusHandle_t,
            table_name: *const c_char,
        ) -> rbusError_t {
            if handle.is_null() {
                return rbusError_t::RBUS_ERROR_INVALID_HANDLE;
            }
            if table_name.is_null() {
                return rbusError_t::RBUS_ERROR_INVALID_INPUT;
            }

            // SAFETY: RBus keeps the connection valid for this callback.
            let handle = unsafe { RBusHandleRef::from_raw(&handle) };
            let table_name = unsafe { CStr::from_ptr(table_name) };
            let result = T::sync_rows(handle, table_name);

            match result {
                Ok(_) => rbusError_t::RBUS_ERROR_SUCCESS,
                Err(e) => e.to_raw(),
            }
        }
        let library = RBusLibrary::get();
        let result = unsafe {
            library.rbus_registerDynamicTableSyncHandler(
                *self.handle,
                table_name.as_ptr(),
                Some(handler::<T>),
            )
        };
        RBusError::map(result, || ())
    }

    /// See [`crate::RBusHandle::get`].
    pub fn get<T>(&self, name: &CStr) -> Result<T, RBusGetError>
    where
        T: RBusValueReadable,
    {
        let value = self.get_value(name).map_err(RBusGetError::RBus)?;
        let value = value.get().map_err(RBusGetError::RBusValue)?;
        Ok(value)
    }

    /// See [`crate::RBusHandle::get_value`].
    pub fn get_value(&self, name: &CStr) -> Result<RBusValue, RBusError> {
        let mut value = rbusValue_t::default();
        let library = RBusLibrary::get();
        let result = unsafe { library.rbus_get(*self.handle, name.as_ptr(), &mut value) };

        RBusError::map(result, || RBusValue { handle: value })
    }

    /// See [`crate::RBusHandle::set`].
    pub fn set<T>(&self, name: &CStr, value: &T) -> Result<(), RBusError>
    where
        T: RBusValueWritable + ?Sized,
    {
        self.set_value(name, &RBusValue::from_initialized(value))
    }

    /// See [`crate::RBusHandle::set_value`].
    pub fn set_value(&self, name: &CStr, value: &RBusValue) -> Result<(), RBusError> {
        let library = RBusLibrary::get();
        let result = unsafe {
            library.rbus_set(
                *self.handle,
                name.as_ptr(),
                value.handle,
                std::ptr::null_mut(),
            )
        };
        RBusError::map(result, || ())
    }

    /// See [`crate::RBusHandle::get_table_row_names`].
    pub fn get_table_row_names(&self, table_name: &CStr) -> Result<RBusRowNames<'a>, RBusError> {
        let mut row_info = std::ptr::null_mut();
        let library = RBusLibrary::get();
        let result = unsafe {
            library.rbusTable_getRowNames(*self.handle, table_name.as_ptr(), &mut row_info)
        };
        RBusError::map(result, || RBusRowNames {
            handle: *self,
            root: row_info,
        })
    }

    /// See [`crate::RBusHandle::to_raw`].
    pub fn to_raw(&self) -> *const () {
        (*self.handle).cast()
    }
}

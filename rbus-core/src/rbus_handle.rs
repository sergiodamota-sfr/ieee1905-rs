mod element;
mod error;
mod name;
mod reference;
mod status;

use crate::rbus_library::RBusLibrary;
use crate::rbus_value::RBusValue;
use crate::{RBusValueReadable, RBusValueWritable};
pub use element::*;
pub use error::*;
use name::RBusHandleName;
use rbus_sys::*;
pub use reference::RBusHandleRef;
pub use status::*;
use std::ffi::CStr;
use std::marker::PhantomData;

///
/// An RBus handle which identifies an opened component
///
pub struct RBusHandle {
    pub(crate) handle: rbusHandle_t,
    _name_reservation: RBusHandleName,
}

unsafe impl Send for RBusHandle {}
unsafe impl Sync for RBusHandle {}

impl RBusHandle {
    ///
    /// Open a bus connection for a software component.
    /// Returns RBusError::GeneralError if the shared library cannot be loaded.
    ///
    /// If multiple components share a software process, the first component that
    /// calls this API will establish a new socket connection to the bus broker.
    /// All calls to this API will receive a dedicated bus handle for that component.
    ///
    /// Opening a component name that is already in use returns
    /// RBusError::ComponentNameDuplication without closing the existing connection.
    /// The name can be reused after its handle is dropped.
    ///
    /// Note: This API supports a single component per software process and also
    /// supports multiple components that share a software process. In the case of
    /// multiple components that share a software process, each component must call
    /// this API to open a bus connection. Only a single socket connection is opened
    /// per software process but each component within that process receives a
    /// separate busHandle.
    ///
    /// Used by: All RBus components to begin a connection with the bus.
    ///
    pub fn open(name: &CStr) -> Result<Self, RBusError> {
        RBusLibrary::load().map_err(|_| RBusError::GeneralError)?;
        let reserved_name = RBusHandleName::reserve(name)?;
        let mut handle = rbusHandle_t::default();
        let result = unsafe { RBusLibrary::get().rbus_open(&mut handle, name.as_ptr()) };

        RBusError::map(result, || Self {
            handle,
            _name_reservation: reserved_name,
        })
    }

    ///
    /// Borrow this connection without transferring ownership.
    ///
    pub fn as_ref(&self) -> RBusHandleRef<'_> {
        RBusHandleRef {
            handle: &self.handle,
        }
    }

    ///
    /// A Component uses this API to register one or more named Data Elements
    /// (i.e., parameters and/or event names) that will be accessible/subscribable
    /// by other components. This also registers the callback functions
    /// associated with each data element using the dataElement structure.
    ///
    /// Used by: All components that provide named parameters and/or events that
    /// may be accessed/subscribed by other component(s)
    ///
    pub fn register_data_elements(&self, elements: &[RBusDataElement]) -> Result<(), RBusError> {
        self.as_ref().register_data_elements(elements)
    }

    ///
    /// A Component uses this API to unregister one or more previously
    /// registered Data Elements (i.e., named parameters and/or event names) that
    /// will no longer be accessible / subscribable by other components.
    ///
    /// Used by: All components that provide named parameters and/or events that
    /// may be accessed/subscribed by other component(s)
    ///
    pub fn unregister_data_elements(&self, elements: &[RBusDataElement]) -> Result<(), RBusError> {
        self.as_ref().unregister_data_elements(elements)
    }

    ///
    /// Register a row that the provider has added to its own table.
    ///
    /// This method allows a provider to register a row that it adds to its own table.
    /// A provider can add a row internally without the need to call rbusTable_addRow which would
    /// call the provider's tableAddRow handler. However, in order for consumers to know the row exists,
    /// it must be registered.
    ///
    /// Used by: Any provider that adds a row to its own table.
    ///
    /// # Arguments
    /// * `table_name`    - The name of a table (e.g. "Device.IP.Interface.")
    /// * `instance_id`   - The unique instance number the provider has assigned this row.
    /// * `alias_name`    - An optional name for the new row. Must be unique in the table.
    ///
    pub fn register_table_row(
        &self,
        table_name: &CStr,
        instance_id: u32,
        alias_name: Option<&CStr>,
    ) -> Result<(), RBusError> {
        self.as_ref()
            .register_table_row(table_name, instance_id, alias_name)
    }

    ///
    /// Unregister a row that the provider has removed from its own table.
    ///
    /// The method allows a provider to unregister a row that it removes from its own table.
    /// A provider can remove a row internally without the need to call rbusTable_removeRow which would
    /// call the provider's tableRemoveRow handler. However, in order for consumer to know the row no
    /// longer exists, it must be unregistered.
    ///
    /// Used by: Any provider that removes a row from its own table.
    ///
    /// # Arguments
    /// * `row_name` - The name of a table row (e.g. "Device.IP.Interface.1")
    ///
    pub fn unregister_table_row(&self, row_name: &CStr) -> Result<(), RBusError> {
        self.as_ref().unregister_table_row(row_name)
    }

    ///
    /// Register tableSyncHandler for table element with dynamic rows.
    ///
    /// Used by: Component that wants to register tableSyncHandler that will register/unregister rows of dynamic table.
    ///
    /// # Arguments
    /// * `table_name` - The name of a table (e.g. "Device.IP.Interface.")
    ///
    pub fn register_dynamic_table_sync_handler<T>(&self, table_name: &CStr) -> Result<(), RBusError>
    where
        T: RBusTableSyncHandler,
    {
        self.as_ref()
            .register_dynamic_table_sync_handler::<T>(table_name)
    }

    ///
    /// Get the value of a single parameter.
    ///
    /// Used by: All components that need to get an individual parameter
    ///
    pub fn get<T>(&self, name: &CStr) -> Result<T, RBusGetError>
    where
        T: RBusValueReadable,
    {
        self.as_ref().get(name)
    }

    ///
    /// Get the value of a single parameter.
    ///
    /// Used by: All components that need to get an individual parameter
    ///
    pub fn get_value(&self, name: &CStr) -> Result<RBusValue, RBusError> {
        self.as_ref().get_value(name)
    }

    ///
    /// A component uses this to perform a set operation for a single
    /// explicit parameter and has the option to used delayed (coordinated)
    /// commit commands.
    ///
    /// Used by: All components that need to set an individual parameter
    ///
    pub fn set<T>(&self, name: &CStr, value: &T) -> Result<(), RBusError>
    where
        T: RBusValueWritable + ?Sized,
    {
        self.as_ref().set(name, value)
    }

    ///
    /// A component uses this to perform a set operation for a single
    /// explicit parameter and has the option to used delayed (coordinated)
    /// commit commands.
    ///
    /// Used by: All components that need to set an individual parameter
    ///
    pub fn set_value(&self, name: &CStr, value: &RBusValue) -> Result<(), RBusError> {
        self.as_ref().set_value(name, value)
    }

    ///
    /// Get a list of the row names in a table
    /// This method allows a consumer to get the names of all rows on a table.
    ///
    /// Used by:  Any component that needs to know the rows in a table.
    ///
    /// # Arguments
    /// * `table_name` - The name of a table (e.g. "Device.IP.Interface.")
    ///
    pub fn get_table_row_names(&self, table_name: &CStr) -> Result<RBusRowNames<'_>, RBusError> {
        self.as_ref().get_table_row_names(table_name)
    }

    ///
    /// Returns raw handle
    ///
    pub fn to_raw(&self) -> *const () {
        self.as_ref().to_raw()
    }
}

impl Drop for RBusHandle {
    fn drop(&mut self) {
        unsafe { RBusLibrary::get().rbus_close(self.handle) };
    }
}

pub struct RBusRowName<'a> {
    /// Instance number of the row
    pub instance: u32,
    /// Fully qualified row name
    pub name: &'a CStr,
    /// Alias of the row.
    pub alias: Option<&'a CStr>,
}

pub struct RBusRowNames<'a> {
    handle: RBusHandleRef<'a>,
    root: *mut rbusRowName_t,
}

impl RBusRowNames<'_> {
    /// Iterate over borrowed rows.
    pub fn iter(&self) -> RBusRowNameIter<'_> {
        RBusRowNameIter {
            head: self.root,
            phantom_data: PhantomData,
        }
    }
}

impl Drop for RBusRowNames<'_> {
    fn drop(&mut self) {
        unsafe {
            let library = RBusLibrary::get();
            library.rbusTable_freeRowNames(*self.handle.handle, self.root)
        };
    }
}

pub struct RBusRowNameIter<'a> {
    head: *mut rbusRowName_t,
    phantom_data: PhantomData<&'a rbusRowName_t>,
}

impl<'a> Iterator for RBusRowNameIter<'a> {
    type Item = RBusRowName<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head.is_null() {
            return None;
        }

        // SAFETY: the collection keeps the list and strings valid for 'a.
        unsafe {
            let head = &*self.head;
            self.head = head.next;

            let name = CStr::from_ptr(head.name);
            let alias = match head.alias.is_null() {
                true => None,
                false => Some(CStr::from_ptr(head.alias)),
            };

            Some(RBusRowName {
                instance: head.instNum,
                name,
                alias,
            })
        }
    }
}

//! Set of utilities for working with C++ virtual function tables.
//! 
//! This crate does not use any proc macros; all of the macros use `macro_rules!`.

#![no_std]
#![allow(clippy::tabs_in_doc_comments)]

use ::core::{
	fmt,
	ptr::NonNull,
};

mod macros;

/// Type for virtual function table pointers.
pub type VTablePtr<VTable> = NonNull<VTable>;

/// Type alias for pointers to C++ objects of classes that have virtual function tables,
/// conforming to the Itanium ABI.
/// 
/// # What the definition implies
/// The current definition for this type alias implies that a C++ object with a VTable
/// has a non-null pointer to the VTable as the first field with `repr(C)`.
pub type VtObjectPtr<VTable> = NonNull<VTablePtr<VTable>>;

/// Structure that imitates the layout of a C++ object with a `VTable`.
#[repr(C)]
pub struct VtObject<VTable> {
	/// Invariant: This field always contains a valid pointer to the `VTable` for a C++ class,
	/// as specified by the Itanium ABI.
	vtable: VTablePtr<VTable>,
}

impl<VTable> fmt::Debug for VtObject<VTable> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("VtObject")
			.field("vtable", &self.vtable)
			.finish()
	}
}

impl<VTable> VtObject<VTable> {
	/// Return an immutable reference that represents a C++ object
	/// with a corresponding `VTable`
	/// that is valid for the duration of `'a`.
	/// 
	/// # Safety
	/// The pointer must be valid for the duration of lifetime `'a`.
	pub const unsafe fn from_ptr_const<'a>(ptr: VtObjectPtr<VTable>) -> &'a Self {
		unsafe { ptr.cast().as_ref() }
	}

	/// Return a mutable reference that represents a C++ object
	/// with a corresponding `VTable`
	/// that is valid for the duration of `'a`.
	/// 
	/// # Safety
	/// The pointer must be valid for the duration of lifetime `'a`,
	/// and there must also not be any other mutable references to the same object.
	pub const unsafe fn from_ptr_mut<'a>(ptr: VtObjectPtr<VTable>) -> &'a mut Self {
		unsafe { ptr.cast().as_mut() }
	}

	/// Return a pointer that can be used with C++.
	pub const fn as_ptr(&self) -> VtObjectPtr<VTable> {
		unsafe { VtObjectPtr::new_unchecked(self as *const Self as *mut _) }
	}

	/// Return a pointer to the object's `VTable`.
	pub const fn vtable_ptr(&self) -> VTablePtr<VTable> {
		self.vtable
	}

	/// Return a reference to the object's `VTable`.
	pub const fn vtable(&self) -> &VTable {
		// SAFETY: `vtable` is always valid.
		unsafe { self.vtable.as_ref() }
	}

	/// Return a mutable reference to the object's `VTable`.
	/// 
	/// # Safety
	/// The `VTable` is usually not intended to be modified,
	/// and all sorts of Undefined Behavior may arise from its modification.
	pub const unsafe fn vtable_mut(&mut self) -> &mut VTable {
		// SAFETY: `vtable` is always valid.
		unsafe { self.vtable.as_mut() }
	}
}

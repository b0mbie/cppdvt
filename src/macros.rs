/// Macro to generate VTable structs with a domain-specific language.
/// 
/// Generated structs have `#[repr(C)]` applied to them, and functions defined
/// within them are `extern "C-unwind"` on non-Windows x86
/// and `extern "thiscall-unwind"` on Windows x86 targets.
/// 
/// VTables can be used with [`virtual_call!`].
/// 
/// # Function order
/// VTable functions must be defined in order of their appearance in the header
/// file of the class they are defined in.
/// 
/// Do keep in mind that on `cfg(not(windows))`, there are *two*
/// virtual destructors.
/// 
/// # Examples
/// A simple VTable can be defined like this:
/// ```
/// use cppdvt::vtable;
/// 
/// // Assume that `Pet` is a class with a VTable that has the method
/// // `Pet::speak()`, which returns nothing.
/// 
/// vtable! {
/// 	/// VTable for `Pet`.
/// 	#[derive(Debug)]
/// 	pub(crate) PetVt {
/// 		pub fn speak();
/// 	}
/// }
/// ```
/// 
/// A VTable for a pre-defined type (for example, if there is inheritance) can
/// be defined by making the `this` a different type:
/// ```
/// use core::ffi::c_char;
/// 
/// use cppdvt::vtable;
/// 
/// // Assume that `Pet` is a class with a VTable that has the method
/// // `Pet::speak()`, which returns nothing, the method `Pet::kind()`, which
/// // returns the kind of pet it is, and the method `Pet::name()`, which
/// // returns the name of the pet.
/// 
/// /// Enumeration of possible pets as defined in C.
/// #[repr(C)]
/// pub enum PetKind {
/// 	Lizard = 0,
/// 	Snake = 1,
/// }
/// 
/// vtable! {
/// 	/// VTable for `Pet`.
/// 	pub PetVt {
/// 		#[doc = "Make the Pet call their callback for speaking."]
/// 		pub fn speak() -> ();
/// 		pub fn kind() -> PetKind;
/// 		pub fn name() -> *const c_char;
/// 	}
/// }
/// 
/// vtable! {
/// 	/// VTable extension for `Lizard`, which extends from `PetVt`.
/// 	LizardVtExt {
/// 		pub fn derp();
/// 	}
/// }
/// 
/// /// VTable for `Lizard`.
/// #[repr(C)]
/// pub struct LizardVt {
/// 	base: PetVt,
/// 	lizard: LizardVtExt
/// }
/// 
/// vtable! {
/// 	/// VTable extension for `Snake`, which extends from `PetVt`.
/// 	SnakeVtExt {
/// 		pub fn curl(outer_radius: u32);
/// 	}
/// }
/// 
/// /// VTable for `Snake`.
/// #[repr(C)]
/// pub struct SnakeVt {
/// 	base: PetVt,
/// 	snake: SnakeVtExt
/// }
/// ```
#[macro_export]
macro_rules! vtable {
	{
		$(#[$vt_attr:meta])*
		$vt_vis:vis $vt_name:ident for $vt_this:ty {
			$(
				$(#[$fn_attr:meta])*
				$fn_vis:vis fn $fn_name:ident($($fn_param:tt)*) $(-> $fn_ret:ty)?;
			)*
		}
	} => {
		$(#[$vt_attr])*
		#[repr(C)]
		$vt_vis struct $vt_name {
			$(
				#[cfg(all(windows, target_arch = "x86"))]
				$(#[$fn_attr])*
				$fn_vis $fn_name:
					unsafe extern "thiscall-unwind" fn (
						this: $vt_this, $($fn_param)*
					) $(-> $fn_ret)?,
				#[cfg(not(all(windows, target_arch = "x86")))]
				$(#[$fn_attr])*
				$fn_vis $fn_name:
					unsafe extern "C-unwind" fn (
						this: $vt_this, $($fn_param)*
					) $(-> $fn_ret)?,
			)*
		}
	};

	(
		$(#[$vt_attr:meta])*
		$vt_vis:vis $vt_name:ident {
			$(
				$(#[$fn_attr:meta])*
				$fn_vis:vis fn $fn_name:ident($($fn_param:tt)*) $(-> $fn_ret:ty)?;
			)*
		}
	) => {
		$crate::vtable! {
			$(#[$vt_attr])*
			$vt_vis $vt_name for $crate::VtObjectPtr<$vt_name> {
				$(
					$(#[$fn_attr])*
					$fn_vis fn $fn_name($($fn_param)*) $(-> $fn_ret)?;
				)*
			}
		}
	};
}

/// Given `$type` is a VTable type and `Self` has all of the virtual methods for
/// that VTable with the same name, create a new VTable with those methods.
/// 
/// Optionally, you can specify `for $self:ident`, where `$self` is the type to
/// use instead of `Self`.
#[macro_export]
macro_rules! new_vtable_self {
	(
		$type:ident {
			$(
				$(#[$set_attr:meta])*
				$func_name:ident
			),*
		}
	) => {
		$type {
			$(
                $(#[$set_attr])*
				$func_name: Self::$func_name
			),*
		}
	};

	(
		$type:ident for $self:ident {
			$(
				$(#[$set_attr:meta])*
				$func_name:ident
			),*
		}
	) => {
		$type {
			$(
                $(#[$set_attr])*
				$func_name: $self::$func_name
			),*
		}
	};
}

/// Convert the pointer `$this` to a probably-`mut` reference to `Self`.
#[macro_export]
macro_rules! this_to_self {
	(mut $this:expr) => {
		$this.cast::<Self>().as_mut()
	};

	(ref $this:expr) => {
		$this.cast::<Self>().as_ref()
	};
}

/// Given an invokation of the form `vt_object => name(...)`,
/// invoke the virtual method `name`
/// of the [`VtObject`](crate::VtObject) `vt_object`
/// with the specified arguments, if any.
#[macro_export]
macro_rules! virtual_call {
	($vt_object:expr => $name:ident($($arg:tt)*)) => {{
		let vt_object = &$vt_object;
		($crate::VtObject::vtable(vt_object).$name)($crate::VtObject::as_ptr(vt_object), $($arg)*)
	}};
}

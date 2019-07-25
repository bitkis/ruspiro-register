/*********************************************************************************************************************** 
 * Copyright (c) 2019 by the authors
 * 
 * Author: André Borrmann 
 * License: Appache License 2.0
 **********************************************************************************************************************/

//! # Register abstraction implementation
//! 
//! The provided implementation details of the register access abstraction are used by the corresponding macros
//! of this crate. It is preferred to use the macros to properly define the registers to be used.

use core::ptr::{read_volatile, write_volatile};
use core::ops::{BitOr, BitAnd, Not, Shl, Shr};

/// This trait is used to describe the register size/length as type specifier. The trait is only implemented for the
/// internal types **u8**, **u16**, **u32** and **u64** to ensure safe register access sizes with compile time checking
pub trait RegisterType: 
    Copy + 
    Clone +
    BitOr<Output=Self> +
    BitAnd<Output=Self> + 
    Not<Output=Self> +
    Shl<Self, Output=Self> +
    Shr<Self, Output=Self> { }

// Internal macro to ease the assignment of the custom trait to supported register sizes
#[doc(hidden)]
macro_rules! registertype_impl {
    // invoke the macro for a given type t as often as types are provided when invoking the macro
    ($( $t:ty ),*) => ($(
        impl RegisterType for $t { }        
    )*)
}

// implement the type trait for specific unsigned types to enable only those register types/sizes
registertype_impl![u8, u16, u32, u64];

/// This struct allows read only access to a register.
#[derive(Clone)]
pub struct ReadOnly<T: RegisterType> {
    ptr: *mut T, // base address for the register
}

/// This struct allows write only access to a register.
#[derive(Clone)]
pub struct WriteOnly<T: RegisterType> {
    ptr: *mut T, // base address for the register
}

/// This struct allows read/write access to a register.
#[derive(Clone)]
pub struct ReadWrite<T: RegisterType> {
    ptr: *mut T, // base address for the register
}

/*************** internal used macros to ease implementation ******************/
macro_rules! registernew_impl {
    () => (
        /// Create a new instance of the register access struct.
        pub const fn new(addr: u32) -> Self {
            Self { ptr: addr as *mut T }
        }
    )
}

macro_rules! registerget_impl {
    () => (
        /// Read raw content of a register.
        #[inline]
        pub fn get(&self) -> T {
            unsafe { read_volatile(self.ptr) }
        }

        /// Read the value of a specific register field
        #[inline]
        pub fn read(&self, field: RegisterField<T>) -> T {
            let val = self.get();
            (val & field.mask) >> field.shift
        }
    )
}

macro_rules! registerset_impl {
    () => (
        /// Write raw content value to the register.
        #[inline]
        pub fn set(&self, value: T) {
            unsafe { write_volatile(self.ptr, value) }
        }

        /// Write the value of a specific register field
        #[inline]
        pub fn write(&self, field: RegisterField<T>, value: T) {
            let val = (value & field.mask) << field.shift;
            self.set(val);
        }
    )
}

impl<T: RegisterType> ReadOnly<T> {
    registernew_impl!();
    registerget_impl!();
}

impl<T: RegisterType> WriteOnly<T> {
    registernew_impl!();
    registerset_impl!();
}

impl<T: RegisterType> ReadWrite<T> {
    registernew_impl!();
    registerget_impl!();
    registerset_impl!();

    /// Udate a register field with a given value
    pub fn modify(&self, field: RegisterField<T>, value: T) -> T {

        let old_val = self.get();
        let new_val = (old_val & !field.mask) | (value << field.shift);
        self.set(new_val);
        
        new_val
    }
}

/// Definition of a field contained inside of a register. Each field is defined by a mask and the bit shift value
/// when constructing the field definition the stored mask is already shifted by the shift value
#[derive(Copy, Clone)]
pub struct RegisterField<T: RegisterType> {
    mask: T,
    shift: T,
}

// Internal helper macro to implement the ```RegisterField```struct for all relevant basic types
#[doc(hidden)]
macro_rules! registerfield_impl {
    ($($t:ty),*) => ($(
        impl RegisterField<$t> {
            pub const fn new(mask: $t, shift: $t) -> RegisterField<$t> {
                Self {
                    mask: mask << shift,
                    shift: shift,
                }
            }
        }
    )*);
}

registerfield_impl![u8, u16, u32, u64];
#![no_std]
#![feature(adt_const_params)]
#![allow(incomplete_features)]

use core::marker::ConstParamTy;

use wdk_sys::{ntddk::KeGetCurrentIrql, APC_LEVEL, DISPATCH_LEVEL, PASSIVE_LEVEL};
pub use wdrf_proc_macros::irql_check;

#[derive(Clone, Copy, Debug, ConstParamTy, PartialEq, Eq)]
pub enum IrqlCompare {
    Eq,
    Less,
    LessEq,
    Greater,
    GreaterEq,
}

#[repr(u32)]
pub enum IrqlLevel {
    Passive = PASSIVE_LEVEL,
    Apc = APC_LEVEL,
    Dispatch = DISPATCH_LEVEL,
}

#[inline]
pub fn irql_check_compare_and_panic<const C: IrqlCompare>(expected_irql: u32) {
    unsafe {
        let irql = KeGetCurrentIrql() as u32;
        let result = match C {
            IrqlCompare::Eq => irql == expected_irql,
            IrqlCompare::Less => irql < expected_irql,
            IrqlCompare::LessEq => irql <= expected_irql,
            IrqlCompare::Greater => irql > expected_irql,
            IrqlCompare::GreaterEq => irql >= expected_irql,
        };

        if !result {
            panic!("Current irql {irql}, expected: {expected_irql}, compare: {C:?}");
        }
    }
}

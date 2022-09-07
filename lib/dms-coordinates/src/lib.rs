//! Package to manipulate 1D and 3D D°M'S'' coordinates,
//! mainly in navigation applications.   
//! Homepage: <https://github.com/gwbres/dms-coordinates>
pub mod dms;
pub mod cardinal;

pub use crate::{
    dms::DMS,
    cardinal::Cardinal,
};

#[cfg(test)]
#[macro_use]
extern crate assert_float_eq;

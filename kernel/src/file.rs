// Copyright (c) 2020 Alex Chi
//
// This software is released under the MIT License.
// https://opensource.org/licenses/MIT

//! File in core-os including file in filesystem, device, pipe and symbol link

use alloc::boxed::Box;

mod device;
pub use device::*;
mod fsfile;
pub use fsfile::*;

/// File in core-os
pub enum File {
    Device(Box<dyn Device>),
    FsFile(FsFile),
    Pipe
}

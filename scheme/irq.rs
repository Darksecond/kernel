use core::{mem, str};

use arch::interrupt::irq::{ACKS, COUNTS, acknowledge};
use syscall::{Error, Result};
use super::Scheme;

pub struct IrqScheme;

impl Scheme for IrqScheme {
    fn open(&mut self, path: &[u8], _flags: usize) -> Result<usize> {
        let path_str = str::from_utf8(path).or(Err(Error::NoEntry))?;

        let id = path_str.parse::<usize>().or(Err(Error::NoEntry))?;

        if id < COUNTS.lock().len() {
            Ok(id)
        } else {
            Err(Error::NoEntry)
        }
    }

    fn dup(&mut self, file: usize) -> Result<usize> {
        Ok(file)
    }

    fn read(&mut self, file: usize, buffer: &mut [u8]) -> Result<usize> {
        // Ensures that the length of the buffer is larger than the size of a usize
        if buffer.len() >= mem::size_of::<usize>() {
            let ack = ACKS.lock()[file];
            let current = COUNTS.lock()[file];
            if ack != current {
                // Safe if the length of the buffer is larger than the size of a usize
                assert!(buffer.len() >= mem::size_of::<usize>());
                unsafe { *(buffer.as_mut_ptr() as *mut usize) = current; }
                Ok(mem::size_of::<usize>())
            } else {
                Ok(0)
            }
        } else {
            Err(Error::InvalidValue)
        }
    }

    fn write(&mut self, file: usize, buffer: &[u8]) -> Result<usize> {
        if buffer.len() >= mem::size_of::<usize>() {
            assert!(buffer.len() >= mem::size_of::<usize>());
            let ack = unsafe { *(buffer.as_ptr() as *const usize) };
            let current = COUNTS.lock()[file];
            if ack == current {
                ACKS.lock()[file] = ack;
                unsafe { acknowledge(file); }
                Ok(mem::size_of::<usize>())
            } else {
                Ok(0)
            }
        } else {
            Err(Error::InvalidValue)
        }
    }

    fn fsync(&mut self, _file: usize) -> Result<()> {
        Ok(())
    }

    fn close(&mut self, _file: usize) -> Result<()> {
        Ok(())
    }
}

use super::{Env, Read, StringPtr};
use wasmer::{FunctionEnvMut, MemoryAccessError, MemoryView};

// if get_string throws an exception abort for some reason is being called
pub fn abort(
    ctx: FunctionEnvMut<Env>,
    message: StringPtr,
    filename: StringPtr,
    line: u32,
    col: u32,
) -> Result<(), wasmer::RuntimeError> {
    let memory = ctx.data().memory.as_ref().expect("mem??").clone();
    let message_ = message
        .read(&memory, &ctx)
        .map_err(|e| wasmer::RuntimeError::new(e.to_string()))?;
    let filename_ = filename
        .read(&memory, &ctx)
        .map_err(|e| wasmer::RuntimeError::new(e.to_string()))?;
    eprintln!("Error: {} at {}:{} col: {}", message_, filename_, line, col);
    Ok(())
}

macro_rules! export_asr {
    ($func_name:ident, $env:expr) => {
        match $env.$func_name.as_ref() {
            Some(res) => res,
            _ => anyhow::bail!("Failed to get func in env"),
        }
    };
}

macro_rules! export_mem {
    ($env:expr) => {
        match $env.memory.as_ref() {
            Some(res) => res,
            _ => anyhow::bail!("No memory in env"),
        }
    };
}

/// Fails like `WasmSlice::read_slice` does when `byte_len` bytes at `offset` do not fit in the
/// guest memory. Readers call it before allocating the host buffer: the length comes from a
/// guest-controlled header, so allocating first lets a small guest request gigabytes.
pub(crate) fn check_guest_bounds(
    memory_view: &MemoryView,
    offset: u32,
    byte_len: u64,
) -> Result<(), MemoryAccessError> {
    let end = u64::from(offset)
        .checked_add(byte_len)
        .ok_or(MemoryAccessError::Overflow)?;
    if end > memory_view.data_size() {
        return Err(MemoryAccessError::HeapOutOfBounds);
    }
    Ok(())
}

pub(crate) use export_asr;
pub(crate) use export_mem;

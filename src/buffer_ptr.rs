use super::{Env, Memory, Read, Write};
use std::convert::{TryFrom, TryInto};
use wasmer::{FromToNativeWasmType, Store, Value, WasmPtr};

use crate::tools::export_asr;

#[derive(Clone, Copy)]
pub struct BufferPtr(WasmPtr<u8>);

impl BufferPtr {
    pub fn new(offset: u32) -> Self {
        Self(WasmPtr::new(offset))
    }
    pub fn offset(&self) -> u32 {
        self.0.offset()
    }
}

unsafe impl FromToNativeWasmType for BufferPtr {
    type Native = i32;
    fn to_native(self) -> Self::Native {
        self.offset() as i32
    }
    fn from_native(n: Self::Native) -> Self {
        Self::new(n as u32)
    }
}

impl Read<Vec<u8>> for BufferPtr {
    fn read(&self, memory: &Memory, store: &Store) -> anyhow::Result<Vec<u8>> {
        let size = self.size(memory, store)?;

        let memory_view = memory.view(store);
        let wasm_slice_ = self.0.slice(&memory_view, size);

        if let Ok(wasm_slice) = wasm_slice_ {
            let mut res = Vec::with_capacity(size as usize);
            wasm_slice.read_slice(&mut res)?;
            Ok(res)
        } else {
            anyhow::bail!("Wrong offset: can't read buf")
        }
    }

    fn size(&self, memory: &Memory, store: &Store) -> anyhow::Result<u32> {
        size(self.0.offset(), memory, store)
    }
}

impl Write<Vec<u8>> for BufferPtr {
    fn alloc(value: &Vec<u8>, env: &Env, store: &mut Store) -> anyhow::Result<Box<BufferPtr>> {
        let new = export_asr!(fn_new, env);
        let size = i32::try_from(value.len())?;

        let offset = u32::try_from(
            if let Some(value) = new.call(store, &[Value::I32(size), Value::I32(0)])?.get(0) {
                match value.i32() {
                    Some(offset) => offset,
                    _ => anyhow::bail!("Unable to allocate value"),
                }
            } else {
                anyhow::bail!("Unable to allocate value")
            },
        )?;
        write_buffer(offset, value, env, store)?;
        Ok(Box::new(BufferPtr::new(offset)))
    }

    fn write(&mut self, value: &Vec<u8>, env: &Env, store: &mut Store) -> anyhow::Result<Box<Self>> {
        let memory = &env.memory;
        let prev_size = size(self.offset(), memory, store)?;
        let new_size = u32::try_from(value.len())?;
        if prev_size == new_size {
            write_buffer(self.offset(), value, env, store)?;
            Ok(Box::new(*self))
        } else {
            // unpin old ptr
            let unpin = export_asr!(fn_pin, env);
            unpin.call(store, &[Value::I32(self.offset().try_into()?)])?;

            // collect
            let collect = export_asr!(fn_collect, env);
            collect.call(store, &[])?;

            // alloc with new size
            BufferPtr::alloc(value, env, store)
        }
    }

    fn free(self, _env: &Env, store: &mut Store) -> anyhow::Result<()> {
        todo!("Release the memory from this string")
    }
}

fn write_buffer(offset: u32, value: &[u8], env: &Env, store: &Store) -> anyhow::Result<()> {
    /*
    let view = match env.memory.get_ref() {
        Some(mem) => mem.view::<u8>(),
        _ => anyhow::bail!("Uninitialized memory"),
    };
    */
    let view = env.memory.view(store);

    if cfg!(feature = "no_thread") {
        todo!()
        /*
        let subarray_view = view.subarray(offset, offset + (value.len() as u32));
        // copy_from is unsafe because the caller will need to make sure there are no data races
        // when copying memory into the view.
        unsafe {
            subarray_view.copy_from(value);
        }
        */
    } else {
        /*
        let from = usize::try_from(offset)?;
        for (bytes, cell) in value.iter().zip(view[from..from + value.len()].iter()) {
            cell.set(*bytes);
        }
        */

        let from = u64::from(offset);
        view.write(from, value)?;
    }

    Ok(())
}

fn size(offset: u32, memory: &Memory, store: &Store) -> anyhow::Result<u32> {
    if offset < 4 {
        anyhow::bail!("Wrong offset: less than 2")
    }
    // read -4 offset
    // https://www.assemblyscript.org/runtime.html#memory-layout
    /*
    if let Some(cell) = memory.view(store).get(offset as usize / (32 / 8) - 1) {
        Ok(cell.get())
    } else {
        anyhow::bail!("Wrong offset: can't read size")
    }
    */

    let mut size_ = Vec::with_capacity(4);
    memory.view(store).read(offset as u64 / (32 / 8) -1, &mut size_[..])?;
    Ok(u32::from_ne_bytes(size_.try_into().unwrap()))

}

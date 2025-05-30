use super::{Env, Read, StringPtr};
use wasmer::FunctionEnvMut;

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

pub(crate) use export_asr;
pub(crate) use export_mem;

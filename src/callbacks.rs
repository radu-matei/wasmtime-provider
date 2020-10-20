use std::sync::Arc;
use wapc::ModuleState;
use wasmtime::Memory;
use wasmtime::{Caller, Func, FuncType, Store, Val, ValType};

pub(crate) fn guest_request_func(store: &Store, host: Arc<ModuleState>) -> Func {
    let callback_type = FuncType::new(Box::new([ValType::I32, ValType::I32]), Box::new([]));
    Func::new(
        store,
        callback_type,
        move |caller: Caller, params, _results| {
            let op_ptr = params[0].i32();
            let ptr = params[1].i32();

            let invocation = host.get_guest_request();
            let memory = get_caller_memory(&caller).unwrap();
            if let Some(inv) = invocation {
                write_bytes_to_memory(memory.clone(), ptr.unwrap(), &inv.msg);
                write_bytes_to_memory(memory, op_ptr.unwrap(), &inv.operation.as_bytes());
            }
            Ok(())
        },
    )
}

pub(crate) fn console_log_func(store: &Store, host: Arc<ModuleState>) -> Func {
    let callback_type = FuncType::new(Box::new([ValType::I32, ValType::I32]), Box::new([]));

    Func::new(
        store,
        callback_type,
        move |caller, params: &[Val], _results: &mut [Val]| {
            let ptr = params[0].i32();
            let len = params[1].i32();
            let memory = get_caller_memory(&caller).unwrap();
            let vec = get_vec_from_memory(memory, ptr.unwrap(), len.unwrap());

            let msg = std::str::from_utf8(&vec).unwrap();

            host.do_console_log(msg);
            Ok(())
        },
    )
}

pub(crate) fn host_call_func(store: &Store, host: Arc<ModuleState>) -> Func {
    let callback_type = FuncType::new(
        Box::new([
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
            ValType::I32,
        ]),
        Box::new([ValType::I32]),
    );
    Func::new(
        store,
        callback_type,
        move |caller: Caller, params: &[Val], results: &mut [Val]| {
            /*let id = {
                let mut state = state.borrow_mut();
                state.host_response = None;
                state.host_error = None;
                state.id
            }; */
            let memory = get_caller_memory(&caller).unwrap();

            let bd_ptr = params[0].i32();
            let bd_len = params[1].i32();
            let ns_ptr = params[2].i32();
            let ns_len = params[3].i32();
            let op_ptr = params[4].i32();
            let op_len = params[5].i32();
            let ptr = params[6].i32();
            let len = params[7].i32();

            let vec = get_vec_from_memory(memory.clone(), ptr.unwrap(), len.unwrap());
            let bd_vec = get_vec_from_memory(memory.clone(), bd_ptr.unwrap(), bd_len.unwrap());
            let bd = std::str::from_utf8(&bd_vec).unwrap();
            let ns_vec = get_vec_from_memory(memory.clone(), ns_ptr.unwrap(), ns_len.unwrap());
            let ns = std::str::from_utf8(&ns_vec).unwrap();
            let op_vec = get_vec_from_memory(memory, op_ptr.unwrap(), op_len.unwrap());
            let op = std::str::from_utf8(&op_vec).unwrap();
            //trace!("Guest {} invoking host operation", id, op);
            let result = host.do_host_call(bd, ns, op, &vec);
            if let Ok(r) = result {
                results[0] = Val::I32(r);
            }
            Ok(())
        },
    )
}

pub(crate) fn host_response_func(store: &Store, host: Arc<ModuleState>) -> Func {
    let callback_type = FuncType::new(Box::new([ValType::I32]), Box::new([]));
    Func::new(
        store,
        callback_type,
        move |caller: Caller, params: &[Val], _results: &mut [Val]| {
            if let Some(ref e) = host.get_host_response() {
                let memory = get_caller_memory(&caller).unwrap();
                let ptr = params[0].i32();
                write_bytes_to_memory(memory, ptr.unwrap(), &e);
            }
            Ok(())
        },
    )
}

pub(crate) fn host_response_len_func(store: &Store, host: Arc<ModuleState>) -> Func {
    let callback_type = FuncType::new(Box::new([]), Box::new([ValType::I32]));

    Func::new(
        store,
        callback_type,
        move |_caller: Caller, _params: &[Val], results: &mut [Val]| {
            results[0] = Val::I32(match host.get_host_response() {
                Some(ref r) => r.len() as _,
                None => 0,
            });
            Ok(())
        },
    )
}

pub(crate) fn guest_response_func(store: &Store, host: Arc<ModuleState>) -> Func {
    let callback_type = FuncType::new(Box::new([ValType::I32, ValType::I32]), Box::new([]));
    Func::new(
        store,
        callback_type,
        move |caller: Caller, params: &[Val], _results: &mut [Val]| {
            let ptr = params[0].i32();
            let len = params[1].i32();
            let memory = get_caller_memory(&caller).unwrap();
            let vec = get_vec_from_memory(memory, ptr.unwrap(), len.unwrap());
            host.set_guest_response(vec);
            Ok(())
        },
    )
}

pub(crate) fn guest_error_func(store: &Store, host: Arc<ModuleState>) -> Func {
    let callback_type = FuncType::new(Box::new([ValType::I32, ValType::I32]), Box::new([]));
    Func::new(
        store,
        callback_type,
        move |caller: Caller, params: &[Val], _results: &mut [Val]| {
            let memory = get_caller_memory(&caller).unwrap();
            let ptr = params[0].i32();
            let len = params[1].i32();

            let vec = get_vec_from_memory(memory, ptr.unwrap(), len.unwrap());
            host.set_guest_error(String::from_utf8(vec).unwrap());
            Ok(())
        },
    )
}

pub(crate) fn host_error_func(store: &Store, host: Arc<ModuleState>) -> Func {
    let callback_type = FuncType::new(Box::new([ValType::I32]), Box::new([]));
    Func::new(
        store,
        callback_type,
        move |caller: Caller, params: &[Val], _results: &mut [Val]| {
            if let Some(ref e) = host.get_host_error() {
                let ptr = params[0].i32();
                let memory = get_caller_memory(&caller).unwrap();
                write_bytes_to_memory(memory, ptr.unwrap(), e.as_bytes());
            }
            Ok(())
        },
    )
}

pub(crate) fn host_error_len_func(store: &Store, host: Arc<ModuleState>) -> Func {
    let callback_type = FuncType::new(Box::new([]), Box::new([ValType::I32]));
    Func::new(
        store,
        callback_type,
        move |_caller: Caller, _params: &[Val], results: &mut [Val]| {
            results[0] = Val::I32(match host.get_host_error() {
                Some(ref e) => e.len() as _,
                None => 0,
            });
            Ok(())
        },
    )
}

fn get_caller_memory(caller: &Caller) -> Result<Memory, anyhow::Error> {
    let memory = caller
        .get_export("memory")
        .map(|e| e.into_memory().unwrap());
    Ok(memory.unwrap())
}

fn get_vec_from_memory(mem: Memory, ptr: i32, len: i32) -> Vec<u8> {
    let data = unsafe { mem.data_unchecked_mut() };
    data[ptr as usize..(ptr + len) as usize]
        .iter()
        .copied()
        .collect()
}

fn write_bytes_to_memory(memory: Memory, ptr: i32, slice: &[u8]) {
    unsafe {
        let raw = memory.data_ptr().offset(ptr as isize);
        raw.copy_from(slice.as_ptr(), slice.len())
    }
}

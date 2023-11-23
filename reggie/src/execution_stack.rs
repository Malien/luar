use std::mem::{align_of, size_of, size_of_val};

use enum_map::{enum_map, EnumMap};

use crate::{
    ids::{BlockID, LocalRegisterID},
    keyed_vec::KeyedVec,
    machine::{CodeBlock, DataType, ProgramCounter},
    meta::{CodeMeta, LocalRegCount},
    LuaString, LuaValue, NativeFunction, TableRef,
};

pub struct ExecutionStack {
    stack: Vec<u8>,
    #[cfg(debug_assertions)]
    frame_sizes: Vec<usize>,
}

pub struct FrameHandle<'a, 'b> {
    frame: &'a mut StackFrame,
    meta: &'b CodeMeta,
}

/// Stack frame layout:
/// ```
/// +-------------------------------------------------------+
/// | return_addr                                           | <- ProgramCounter
/// +-------------------------------------------------------+
/// | Dynamic locals                                        | <- LuaValue
/// | repeated locals_count[DataType::Dynamic] times        |
/// +-------------------------------------------------------+
/// | Int locals                                            | <- i32
/// | repeated locals_count[DataType::Int] times            |
/// +-------------------------------------------------------+
/// | Float locals                                          | <- f64
/// | repeated locals_count[DataType::Float] times          |
/// +-------------------------------------------------------+
/// | String locals                                         | <- LuaString
/// | repeated locals_count[DataType::String] times         |
/// +-------------------------------------------------------+
/// | Function locals                                       | <- BlockID
/// | repeated locals_count[DataType::Function] times       |
/// +-------------------------------------------------------+
/// | NativeFunction locals                                 | <- Option<NativeFunction>
/// | repeated locals_count[DataType::NativeFunction] times |
/// +-------------------------------------------------------+
/// | Table locals                                          | <- Option<TableRef>
/// | repeated locals_count[DataType::Table] times          |
/// +-------------------------------------------------------+
/// ```
/// locals_count is stored in CodeMeta of the associated function.
/// StackFrame is !Sized, since the size of locals is not known at compile time.
#[repr(C)]
struct StackFrame {
    return_addr: AlignedPC,
    locals: [u8],
}

#[repr(align(8))]
struct AlignedPC(ProgramCounter);

fn value_sizes() -> EnumMap<DataType, usize> {
    enum_map! {
        DataType::Dynamic => size_of::<LuaValue>(),
        DataType::Int => size_of::<i32>(),
        DataType::Float => size_of::<f64>(),
        // This is a special case, since
        DataType::String => size_of::<LuaString>() + align_of::<u32>(),
        DataType::Function => size_of::<BlockID>(),
        DataType::NativeFunction => size_of::<NativeFunction>(),
        DataType::Table => size_of::<TableRef>(),
    }
}

fn from_raw_parts(base: *mut u8, size: usize) -> *mut StackFrame {
    unsafe {
        let slice = std::slice::from_raw_parts_mut(base, size);
        slice as *mut [u8] as *mut StackFrame
    }
}

/// Deinitializes locals of a given type. Returns a pointer to the end of this local type.
unsafe fn deinit_locals<T>(
    mut base: *mut u8,
    dtype: DataType,
    local_count: &LocalRegCount,
) -> *mut u8 {
    let size = value_sizes()[dtype];
    for _ in 0..local_count[dtype] {
        let target_value = base as *mut T;
        std::ptr::drop_in_place(target_value);
        base = unsafe { base.add(size) };
    }
    return base;
}

impl ExecutionStack {
    fn push<'a: 'b, 'b>(&'a mut self, meta: &'a CodeMeta) -> FrameHandle<'a, 'b> {
        let frame_size = stack_frame_size(meta);
        // SAFETY: We just allocated enough space for the frame. Access to the locals
        //         should be aligned... hopefully. I don't know what I'm doing.
        //         I sticked #[repr(align(8))] on the ProgramCounter, hope, this is enough.
        //         StackFrame is itself always aligned, since I compute the offset.
        let frame = unsafe {
            // base_ptr is always aligned, since I manually allign every frame.
            let base_ptr = self.stack.as_mut_ptr().add(self.stack.len());
            #[cfg(debug_assertions)]
            {
                self.frame_sizes.push(frame_size.aligned);
            }

            // SAFETY: This also effectively does zero-bit initialization.
            //         - Zero-bit LuaValue is LuaValue::Nil (I hope).
            //           There is a test for this.
            //         - Zero-bit i32 is 0.
            //         - Zero-bit f64 is 0.0.
            //         - Zero-bit LuaString is an empty string (since len is 0).
            //           There is a test for this.
            //         - Zero-bit BlockID is 0. Which is fine, who cares.
            //         - Zero-bit Option<NativeFunction> is a null pointer (since Rc uses NonNull).
            //           There is a test for this.
            //         - Zero-bit Option<TableRef> is a null pointer (since Rc uses NonNull).
            //           There is a test for this.
            //         Drop of local values is called on stack pop, clear, and Machine drop.
            //         Dropping uncleared stack will panic.
            self.stack
                .extend(std::iter::repeat(0).take(frame_size.aligned));

            let frame_ptr = from_raw_parts(base_ptr, frame_size.locals);
            &mut *frame_ptr
        };

        FrameHandle { frame, meta }
    }

    fn pop<'a>(&'a mut self, handle: FrameHandle<'a, '_>) {
        let frame_size = size_of_val::<StackFrame>(&handle.frame);
        debug_assert!(self.stack.len() >= frame_size);
        debug_assert!(self.frame_sizes.pop() == Some(frame_size));

        let base_ptr = self.stack.as_mut_ptr();
        let count = &handle.meta.local_count;
        // SAFETY: base_ptr points to the beginning of the correct lua type in the locals.
        //         Order is the same as in the enum_map, and by proxy, as in the StackFrame.
        unsafe {
            let base_ptr = deinit_locals::<LuaValue>(base_ptr, DataType::Dynamic, count);
            let base_ptr = deinit_locals::<i32>(base_ptr, DataType::Int, count);
            let base_ptr = deinit_locals::<f64>(base_ptr, DataType::Float, count);
            let base_ptr = deinit_locals::<LuaString>(base_ptr, DataType::String, count);
            let base_ptr = deinit_locals::<BlockID>(base_ptr, DataType::Function, count);
            let base_ptr = deinit_locals::<NativeFunction>(base_ptr, DataType::NativeFunction, count);
            deinit_locals::<TableRef>(base_ptr, DataType::Table, count);
        }

        // SAFETY: frame_size is the exact size of allocated StackFrame, since it includes the size
        //         of the ProgramCounter, size of locals, and alignment.
        self.stack.truncate(self.stack.len() - frame_size);
    }

    // SAFETY: meta should be of the same CodeBlock as the one that was used to create the top frame.
    unsafe fn restore<'a: 'b, 'b>(&'a mut self, meta: &'a CodeMeta) -> FrameHandle<'a, 'b> {
        let frame_size = stack_frame_size(meta);
        // SAFETY: We just allocated enough space for the frame. Access to the locals
        //         should be aligned... hopefully. I don't know what I'm doing.
        //         I sticked #[repr(align(8))] on the ProgramCounter, hope, this is enough.
        //         StackFrame is itself always aligned, since I compute the offset.
        let frame = unsafe {
            // base_ptr is always aligned, since I manually allign every frame.
            debug_assert!(self.stack.len() >= frame_size.aligned);
            let base_ptr = self
                .stack
                .as_mut_ptr()
                .add(self.stack.len() - frame_size.aligned);
            let frame_ptr = from_raw_parts(base_ptr, frame_size.locals);
            &mut *frame_ptr
        };

        FrameHandle { frame, meta }
    }

    /// Traverses the stack from top to bottom, recovering function addresses along the way. They
    /// are used to retrieve the meta of the function in the stack. Meta is required to determine
    /// stack frame sizes. There should be a starting point, since the stack is never empty. That's
    /// why it is required to pass the meta of the top-level function.
    fn clear<'a, 'b>(
        &'b mut self,
        mut last_meta: &'a CodeMeta,
        code_blocks: &'a KeyedVec<BlockID, CodeBlock>,
    ) {
        let frame_size = stack_frame_size(last_meta);
        debug_assert!(self.stack.len() >= frame_size.aligned);
        // This guy lives for as long as the stack is not popped.
        let frame: &'b mut StackFrame = unsafe {
            // base_ptr is always aligned, since I manually allign every frame.
            let base_ptr = self
                .stack
                .as_mut_ptr()
                .add(self.stack.len() - frame_size.aligned);
            let frame_ptr = from_raw_parts(base_ptr, frame_size.locals);
            &mut *frame_ptr
        };

        loop {
            let handle = FrameHandle {
                frame,
                meta: last_meta,
            };
            if frame_size.aligned == self.stack.len() {
                self.pop(handle);
                break;
            }
            let next_meta = &code_blocks[handle.frame.return_addr.0.block].meta;
            self.pop(handle);
            last_meta = next_meta;
        }
    }
}

struct FrameSize {
    locals: usize,
    aligned: usize,
}

fn stack_frame_size(meta: &CodeMeta) -> FrameSize {
    let sizes = value_sizes();
    let locals_size = meta
        .local_count
        .iter()
        .map(|(dtype, count)| sizes[dtype] * *count as usize)
        .sum::<usize>();
    let raw_size = size_of::<AlignedPC>() + locals_size;
    let overshot = raw_size % align_of::<AlignedPC>();
    let align_offset = if overshot > 0 {
        align_of::<AlignedPC>() - overshot
    } else {
        0
    };
    debug_assert!(align_offset < align_of::<AlignedPC>());
    return FrameSize {
        locals: locals_size,
        aligned: raw_size + align_offset,
    };
}

impl Drop for ExecutionStack {
    fn drop(&mut self) {
        if self.stack.len() > 0 {
            panic!("ExecutionStack was not cleared before drop. This will lead to memory leaks.");
        }
    }
}

/// SAFETY: Only in debug builds, will there be a panic if the register is out of bounds.
///         Otherwise, UB galore. Sorry, not sorry, for not marking these methods as unsafe.
impl<'a, 'b> FrameHandle<'a, 'b> {
    fn get_dyn(&mut self, LocalRegisterID(reg): LocalRegisterID) -> &mut LuaValue {
        debug_assert!(self.meta.local_count[DataType::Dynamic] > reg);
        // SAFETY: The space for locals is calculated correctly from function's meta.
        //         The pointer is aligned, since the frame is aligned.
        unsafe {
            let base_ptr = self.frame.locals.as_mut_ptr();
            let val_ptr = base_ptr.add(reg as usize * size_of::<LuaValue>());
            &mut *(val_ptr as *mut LuaValue)
        }
    }

    fn get_int(&mut self, reg: LocalRegisterID) -> &mut i32 {
        self.get_of_type(DataType::Int, reg)
    }

    fn get_float(&mut self, reg: LocalRegisterID) -> &mut f64 {
        self.get_of_type(DataType::Float, reg)
    }

    fn get_string(&mut self, reg: LocalRegisterID) -> &mut LuaString {
        self.get_of_type(DataType::String, reg)
    }

    fn get_function(&mut self, reg: LocalRegisterID) -> &mut BlockID {
        self.get_of_type(DataType::Function, reg)
    }

    fn get_native_function(&mut self, reg: LocalRegisterID) -> &mut NativeFunction {
        self.get_of_type(DataType::NativeFunction, reg)
    }

    fn get_table(&mut self, reg: LocalRegisterID) -> &mut TableRef {
        self.get_of_type(DataType::Table, reg)
    }

    fn get_of_type<T>(
        &mut self,
        target_dtype: DataType,
        LocalRegisterID(reg): LocalRegisterID,
    ) -> &mut T {
        debug_assert!(self.meta.local_count[target_dtype] > reg);
        let base_ptr = self.base_ptr_for_type(target_dtype);
        let offset = value_sizes()[target_dtype] * reg as usize;
        let target_value = unsafe { base_ptr.add(offset) } as *mut T;
        return unsafe { &mut *target_value };
    }

    fn base_ptr_for_type(&mut self, target_dtype: DataType) -> *mut u8 {
        let mut base_ptr = self.frame.locals.as_mut_ptr();

        // SAFETY: Iteration order of enum_map is guaranteed to be the same as the
        //         order of the enum variants. The layout of locals is correspondingly
        //         the same as the order of the enum variants.
        //         Alignment between locals should be ok. Fingers crossed.
        for (dtype, size) in value_sizes() {
            if dtype != target_dtype {
                base_ptr = unsafe { base_ptr.add(size * self.meta.local_count[dtype] as usize) };
            } else {
                return base_ptr;
            }
        }
        unreachable!("enum_map always contains all of the DataType variants");
    }
}

#[cfg(test)]
mod test {
    use std::mem::size_of;

    use crate::{LuaString, LuaValue, NativeFunction, TableRef};

    #[test]
    fn zero_bit_initialized_lua_value_is_nil() {
        let zeros = [0u8; size_of::<LuaValue>()];
        let zero_value: LuaValue = unsafe { std::mem::transmute(zeros) };
        assert_eq!(zero_value, LuaValue::Nil);
    }

    #[test]
    fn zero_bit_initialized_lua_string_is_empty() {
        let zeros = [0u8; size_of::<LuaString>()];
        let zero_value: LuaString = unsafe { std::mem::transmute(zeros) };
        assert_eq!(zero_value, "");
    }

    #[test]
    fn zero_bit_initialized_native_function_is_null() {
        let zeros = [0u8; size_of::<Option<NativeFunction>>()];
        let zero_value: Option<NativeFunction> = unsafe { std::mem::transmute(zeros) };
        assert_eq!(zero_value, None);
    }

    #[test]
    fn zero_bit_initialized_table_ref_is_null() {
        let zeros = [0u8; size_of::<Option<TableRef>>()];
        let zero_value: Option<TableRef> = unsafe { std::mem::transmute(zeros) };
        assert_eq!(zero_value, None);
    }
}

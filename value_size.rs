use std::cell::RefCell;
use std::ptr::NonNull;
use std::rc::Rc;

enum LuaValue {
    Nil,
    Int(i32),
    Float(f64),
    String(LuaString),
    NativeFunction(NativeFunction),
    Function(BlockID),
    Table(TableRef),
}

#[repr(packed(4))]
pub struct LuaString {
    len: u32, 
    block: [u8; 8],
}


struct StrBlockHeader {
    refcount: usize
}

struct StrBlock {
    header: StrBlockHeader,
    data: str,
}

fn main() {
    println!("f64: {}\t\t\tOption<f64>: {}", std::mem::size_of::<f64>(), std::mem::size_of::<Option<f64>>());
    println!("LuaValue: {}\t\tOption<LuaValue>: {}", std::mem::size_of::<LuaValue>(), std::mem::size_of::<Option<LuaValue>>());
    println!("LuaString: {}\t\tOption<LuaString>: {}", std::mem::size_of::<LuaString>(), std::mem::size_of::<Option<LuaString>>());
    println!("String: {}\t\tOption<String>: {}", std::mem::size_of::<String>(), std::mem::size_of::<Option<String>>());
    println!("TableRef: {}\t\tOption<TableRef>: {}", std::mem::size_of::<TableRef>(), std::mem::size_of::<Option<TableRef>>());
    println!("NativeFunction: {}\tOption<NativeFunction>: {}", std::mem::size_of::<NativeFunction>(), std::mem::size_of::<Option<NativeFunction>>());

    println!("[LuaString;2]: {}", std::mem::size_of::<[LuaString; 2]>());
}

pub struct NativeFunction(pub(crate) Rc<NativeFunctionKind>);

pub(crate) enum NativeFunctionKind {
    Dyn(Box<dyn NativeFunctionCallable>),
    // OverloadSet(OverloadSet),
}

trait NativeFunctionCallable {
    fn call(&self);
}

struct BlockID(u32);

pub struct TableValue {
}

pub struct TableRef(Rc<RefCell<TableValue>>);

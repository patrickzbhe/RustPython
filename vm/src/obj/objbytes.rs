use super::super::pyobject::{
    AttributeProtocol, PyContext, PyFuncArgs, PyObject, PyObjectKind, PyObjectRef, PyResult,
    TypeProtocol,
};
use super::super::vm::VirtualMachine;
use super::objint;
use super::objtype;
use num_bigint::ToBigInt;
use num_traits::ToPrimitive;
use std::cell::Ref;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
// Binary data support

// Fill bytes class methods:
pub fn init(context: &PyContext) {
    let ref bytes_type = context.bytes_type;
    bytes_type.set_attr("__eq__", context.new_rustfunc(bytes_eq));
    bytes_type.set_attr("__hash__", context.new_rustfunc(bytes_hash));
    bytes_type.set_attr("__new__", context.new_rustfunc(bytes_new));
    bytes_type.set_attr("__repr__", context.new_rustfunc(bytes_repr));
}

fn bytes_new(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(cls, None)],
        optional = [(val_option, None)]
    );
    if !objtype::issubclass(cls, &vm.ctx.bytes_type()) {
        return Err(vm.new_type_error(format!("{:?} is not a subtype of bytes", cls)));
    }

    // Create bytes data:
    let value = if let Some(ival) = val_option {
        let elements = vm.extract_elements(ival)?;
        let mut data_bytes = vec![];
        for elem in elements.iter() {
            let v = objint::to_int(vm, elem, 10)?;
            data_bytes.push(v.to_u8().unwrap());
        }
        data_bytes
    // return Err(vm.new_type_error("Cannot construct bytes".to_string()));
    } else {
        vec![]
    };

    Ok(PyObject::new(
        PyObjectKind::Bytes { value: value },
        cls.clone(),
    ))
}

fn bytes_eq(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(
        vm,
        args,
        required = [(a, Some(vm.ctx.bytes_type())), (b, None)]
    );

    let result = if objtype::isinstance(b, &vm.ctx.bytes_type()) {
        get_value(a).to_vec() == get_value(b).to_vec()
    } else {
        false
    };
    Ok(vm.ctx.new_bool(result))
}

fn bytes_hash(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(zelf, Some(vm.ctx.bytes_type()))]);
    let data = get_value(zelf);
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    let hash = hasher.finish();
    Ok(vm.ctx.new_int(hash.to_bigint().unwrap()))
}

pub fn get_value<'a>(obj: &'a PyObjectRef) -> impl Deref<Target = Vec<u8>> + 'a {
    Ref::map(obj.borrow(), |py_obj| {
        if let PyObjectKind::Bytes { ref value } = py_obj.kind {
            value
        } else {
            panic!("Inner error getting int {:?}", obj);
        }
    })
}

fn bytes_repr(vm: &mut VirtualMachine, args: PyFuncArgs) -> PyResult {
    arg_check!(vm, args, required = [(obj, Some(vm.ctx.bytes_type()))]);
    let data = get_value(obj);
    let data: Vec<String> = data.iter().map(|b| format!("\\x{:02x}", b)).collect();
    let data = data.join("");
    Ok(vm.new_str(format!("b'{}'", data)))
}

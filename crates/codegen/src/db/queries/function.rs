use std::rc::Rc;

use fe_mir::ir::{FunctionBody, FunctionId, FunctionSignature};

use crate::{db::CodegenDb, yul::legalize};

pub fn legalized_signature(db: &dyn CodegenDb, function: FunctionId) -> Rc<FunctionSignature> {
    let mut sig = function.signature(db.upcast()).as_ref().clone();
    legalize::legalize_func_signature(db, &mut sig);
    sig.into()
}

pub fn legalized_body(db: &dyn CodegenDb, function: FunctionId) -> Rc<FunctionBody> {
    let mut body = function.body(db.upcast()).as_ref().clone();
    legalize::legalize_func_body(db, &mut body);
    body.into()
}

pub fn symbol_name(db: &dyn CodegenDb, function: FunctionId) -> Rc<String> {
    let signature = function.signature(db.upcast());
    // Just a quick and dirty hack to handle monomorphized functions.
    let type_suffix = signature.params.iter().fold(String::new(), |mut acc, x| {
        acc.push_str(&x.ty.0.to_string());
        acc
    });

    let module = signature.module_id;
    let module_name = module.name(db.upcast());
    let func_name = function.name_with_class(db.upcast()).replace("::", "$");

    format!("{}${}_{}", module_name, func_name, type_suffix).into()
}

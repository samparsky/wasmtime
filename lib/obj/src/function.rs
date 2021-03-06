use cranelift_codegen::settings;
use cranelift_codegen::settings::Configurable;
use cranelift_entity::EntityRef;
use faerie::{Artifact, Decl, Link};
use wasmtime_environ::{Compilation, Module, RelocationTarget, Relocations};

/// Defines module functions
pub fn declare_functions(
    obj: &mut Artifact,
    module: &Module,
    relocations: &Relocations,
) -> Result<(), String> {
    for (i, _function_relocs) in relocations.iter().rev() {
        let func_index = module.func_index(i);
        let string_name = format!("_wasm_function_{}", func_index.index());
        obj.declare(string_name, Decl::Function { global: true })
            .map_err(|err| format!("{}", err))?;
    }
    Ok(())
}

/// Emits module functions
pub fn emit_functions(
    obj: &mut Artifact,
    module: &Module,
    compilation: &Compilation,
    relocations: &Relocations,
) -> Result<(), String> {
    debug_assert!(
        module.start_func.is_none()
            || module.start_func.unwrap().index() >= module.imported_funcs.len(),
        "imported start functions not supported yet"
    );

    let mut shared_builder = settings::builder();
    shared_builder
        .enable("enable_verifier")
        .expect("Missing enable_verifier setting");

    for (i, _function_relocs) in relocations.iter() {
        let body = &compilation.functions[i];
        let func_index = module.func_index(i);
        let string_name = format!("_wasm_function_{}", func_index.index());

        obj.define(string_name, body.clone())
            .map_err(|err| format!("{}", err))?;
    }

    for (i, function_relocs) in relocations.iter() {
        let func_index = module.func_index(i);
        let string_name = format!("_wasm_function_{}", func_index.index());
        for r in function_relocs {
            debug_assert_eq!(r.addend, 0);
            match r.reloc_target {
                RelocationTarget::UserFunc(target_index) => {
                    let target_name = format!("_wasm_function_{}", target_index.index());
                    obj.link(Link {
                        from: &string_name,
                        to: &target_name,
                        at: r.offset as u64,
                    })
                    .map_err(|err| format!("{}", err))?;
                }
                _ => panic!("relocations target not supported yet"),
            };
        }
    }

    Ok(())
}

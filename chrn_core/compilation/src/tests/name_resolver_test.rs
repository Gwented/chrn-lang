use super::helpers::*;

/// Namespace-stage diagnostics for a single-module script.
fn ns_diags(text: &str) -> SourceDiagnosticSummary {
    resolve_single_module(text, Stage::Namespace).ns
}

#[test]
fn nameresolver_duplicate_simple_test() {
    // -- NEUTRAL --
    let wrong = "
            let DUPLICATE = 3
            let DUPLICATE = \"Hi\"
            ";

    let diags = ns_diags(wrong);

    assert!(
        !diags.diags.is_empty(),
        "Expected errors from NamespaceResolver"
    );

    let correct = "
                let ORIGINAL = 2 + 2
                let NEW = \"Hallo\"
            ";

    let diags = ns_diags(correct);

    assert!(
        diags.diags.is_empty(),
        "NamespaceResolver should have no errors: {:?}",
        diags
    );

    // -- VAR --
    let wrong = "
            var->
                duplicate: i32
                duplicate: i8
            ";

    // Doing this first since if modules were identified during the parsing stage any
    // syntax error within another module would not be reportable since the parser failed.
    let diags = ns_diags(wrong);

    assert!(
        !diags.diags.is_empty(),
        "Expected errors from NamespaceResolver"
    );

    let correct = "
            var->
                original: u32
                new: i8
            ";

    let diags = ns_diags(correct);

    assert!(
        diags.diags.is_empty(),
        "NamespaceResolver should have no errors: {:?}",
        diags
    );

    // -- NEST --

    let wrong = "
            nest->
                struct Duplicate {}
                struct Duplicate {}
            ";

    let diags = ns_diags(wrong);

    assert!(
        !diags.diags.is_empty(),
        "Expected errors from NamespaceResolver"
    );

    let correct = "
            nest->
                struct Original {}
                struct New {}
            ";

    let diags = ns_diags(correct);

    assert!(
        diags.diags.is_empty(),
        "NamespaceResolver should have no errors: {:?}",
        diags
    );
    //TEST: -- COMPLEX --

    //TEST: -- OVERRIDE --
}

#[test]
fn nameresolver_export_all_compilation_units() {
    let text = "let priv_var = 0
export let export_var = 1

alias priv_alias() = [true]
export alias export_alias() = [true]

var->
    priv_typedef: i32
    export export_typedef: i32

nest->
    struct PrivStruct {}
    export struct ExportStruct {}
    enum PrivEnum {}
    export enum ExportEnum {}
";

    let res = resolve_single_module(text, Stage::Namespace);
    assert_eq!(
        res.ns.err_count(),
        0,
        "NamespaceResolver should have no errors: {:?}",
        res.ns
    );

    let user_mod_id = ModuleId::new(0);

    let find_sym = |name: &str| -> SymbolId {
        let name_id = res
            .interner
            .try_search_str(name)
            .unwrap_or_else(|| panic!("symbol name `{name}` should be interned"));
        res.compiler
            .syms
            .iter()
            .find(|s| s.name_id == name_id)
            .map(|s| s.self_id)
            .unwrap_or_else(|| panic!("symbol `{name}` should be registered"))
    };

    let priv_var = find_sym("priv_var");
    let export_var = find_sym("export_var");
    let priv_alias = find_sym("priv_alias");
    let export_alias = find_sym("export_alias");
    let priv_typedef = find_sym("priv_typedef");
    let export_typedef = find_sym("export_typedef");
    let priv_struct = find_sym("PrivStruct");
    let export_struct = find_sym("ExportStruct");
    let priv_enum = find_sym("PrivEnum");
    let export_enum = find_sym("ExportEnum");

    let exports = &res.compiler.mods[user_mod_id].exports;

    // Confirm that every exportable compilation unit kind is in module.exports when exported
    assert!(
        exports.contains(&export_var),
        "CompilationUnit::Var `export_var` should be registered in module.exports"
    );
    assert!(
        exports.contains(&export_alias),
        "CompilationUnit::Alias `export_alias` should be registered in module.exports"
    );
    assert!(
        exports.contains(&export_typedef),
        "CompilationUnit::TypeDef `export_typedef` should be registered in module.exports"
    );
    assert!(
        exports.contains(&export_struct),
        "CompilationUnit::Struct `ExportStruct` should be registered in module.exports"
    );
    assert!(
        exports.contains(&export_enum),
        "CompilationUnit::Enum `ExportEnum` should be registered in module.exports"
    );

    // Confirm that private compilation units are not registered in module.exports
    assert!(
        !exports.contains(&priv_var),
        "CompilationUnit::Var `priv_var` should not be registered in module.exports"
    );
    assert!(
        !exports.contains(&priv_alias),
        "CompilationUnit::Alias `priv_alias` should not be registered in module.exports"
    );
    assert!(
        !exports.contains(&priv_typedef),
        "CompilationUnit::TypeDef `priv_typedef` should not be registered in module.exports"
    );
    assert!(
        !exports.contains(&priv_struct),
        "CompilationUnit::Struct `PrivStruct` should not be registered in module.exports"
    );
    assert!(
        !exports.contains(&priv_enum),
        "CompilationUnit::Enum `PrivEnum` should not be registered in module.exports"
    );

    assert_eq!(
        exports.len(),
        5,
        "only the 5 exported compilation units should be registered in module.exports"
    );
}

use chrn_utils::{
    chrn_config::{ChrnConfig, chrn_perf::ChrnPerfStage},
    err_codes::ErrorCode,
    id_types::{
        AstId, ConfigRootId, ImplId, ScopeId, SymbolId, TypeId, VariableId, id_tags::TaggedId,
    },
    intern::Intern,
    source_map::source_diagnostic::{
        DiagnosticLevel, SourceDiagnostic, SourceDiagnosticSink, SourceDiagnosticSummary,
        annotations::AnnotationKind,
    },
};

use crate::{
    id_tag_decls::{AliasTag, ConfigRootTag, EnumTag, StructTag, TypeDefTag, VarTag},
    lookup::scopes::scopes_concepts::{Scope, ScopeInfo, ScopeLookupPattern, ScopeType},
    parser::ast::ast_concepts::{
        AbstractAlias, AbstractConfig, AbstractConfigKind, AbstractDecl, AbstractEnum,
        AbstractImpl, AbstractStruct, AbstractTypeDef, AbstractVar, AstConfigMemberMetadataKind,
        Item,
    },
    resolvers::{resolver_env::RegistrationEnv, resolver_state::ResolverState},
    script_compiler::ScriptCompiler,
    semantic::{
        compilation_unit::CompilationUnit,
        hir::{
            hir_concepts::{Type, TypeInfo},
            hir_impls::{
                ConfigRoot, ConfigRootCommon, ConfigRootKind, ConfigRootMetadataKind, ImplHir,
                ImplHirKind,
            },
            hir_symbols::{
                AliasDef, EnumDef, StructDef, Symbol, SymbolKind, SymbolOrigin, TypeDef, VarDef,
                VariableMetadata, VariableState,
            },
        },
    },
};

/// Registers symbols for every front-facing ast item. Members are not accounted for and should
/// be handled by `MemberResolver`.
///
/// This resolver at most reports symbols with the same identifiers in the same scope, but still
/// registers them.
pub struct NamespaceResolver<'a> {
    cfg: &'a mut ChrnConfig,
    interner: &'a Intern,
    compiler: &'a mut ScriptCompiler,
    summary: SourceDiagnosticSummary,
    //NOTE: May handle this differently but ok for now
}

impl NamespaceResolver<'_> {
    /// Instantiation requires that the compiler's state is valid and will panic otherwise
    pub fn new<'a>(
        cfg: &'a mut ChrnConfig,
        interner: &'a Intern,
        compiler: &'a mut ScriptCompiler,
    ) -> NamespaceResolver<'a> {
        // TODO: But this also kind of means that a user CAN'T instantiate a resolver without doing
        // everything at once, or storing the resolver, but then that means the compiler stays
        // borrowed mutably
        //
        // Remove the compiler internal?
        debug_assert_eq!(ResolverState::NAMESPACE, compiler.resolver_state);
        compiler.resolver_state.advance();

        NamespaceResolver {
            cfg,
            interner,
            compiler,
            summary: SourceDiagnosticSummary::default(),
        }
    }

    // Needs the reporter though
    /// Returns the symbols created from the ast nodes within the given module `env` to allow for
    /// module by module compilation at the symbol level.
    pub fn resolve(
        &mut self,
        env: &RegistrationEnv,
    ) -> (Vec<CompilationUnit>, SourceDiagnosticSummary) {
        self.cfg.perf_tracker_mut().start();
        // Storing all symbols created associated with the current module so that compilation
        // doens't have to depend on the ast to keep a coherent understanding of
        let mut comp_units: Vec<CompilationUnit> = Vec::with_capacity(env.ast_info.items.len());

        // Iterates through sections so that it stores the correct scope type associated with the
        // current ast node so it's compilation unit can use said information.
        for abs_sect_opt in &env.ast_info.sections {
            let Some(abs_sect) = abs_sect_opt else {
                continue;
            };

            let scope_type = abs_sect.kind.to_scope_type();

            for ast_id in abs_sect.nodes.iter().cloned() {
                // Maybe opt into section specific processing
                let comp_unit = match &env.ast_info.items[ast_id] {
                    Item::Decl(abs_decl) => match abs_decl {
                        AbstractDecl::TypeDef(abs_typedef) => CompilationUnit::TypeDef(
                            self.register_typedef(abs_typedef, ast_id, scope_type, env),
                        ),
                        AbstractDecl::Struct(abs_struct) => CompilationUnit::Struct(
                            self.register_struct(abs_struct, ast_id, scope_type, env),
                        ),
                        AbstractDecl::Enum(abs_enum) => CompilationUnit::Enum(
                            self.register_enum(abs_enum, ast_id, scope_type, env),
                        ),
                        AbstractDecl::Alias(abs_alias) => CompilationUnit::Alias(
                            self.register_alias(abs_alias, ast_id, scope_type, env),
                        ),
                        AbstractDecl::Var(abs_var) => CompilationUnit::Var(
                            self.register_var(abs_var, ast_id, scope_type, env),
                        ),
                    },
                    Item::Impl(abs_impl) => {
                        let impl_id = match abs_impl {
                            AbstractImpl::Config(abs_cfg) => {
                                self.register_config_root(abs_cfg, ast_id, scope_type, env)
                            }
                        };
                        CompilationUnit::ConfigRoot(impl_id)
                    }
                };

                comp_units.push(comp_unit);
            }
        }

        self.cfg
            .perf_tracker_mut()
            .stop(ChrnPerfStage::NamespaceResolver);
        let mut summary = SourceDiagnosticSummary::default();
        summary.append_summary(&mut self.summary);

        (comp_units, summary)
    }
    // These registrations:
    // - Create a new symbol
    // - Create a new `var`, `nest`, `complex`, or `override` scope if the scope was not pushed yet.
    // - If a symbol with the same identifier as another is in the same scope, it overwrites the last symbol
    // and pushes the diagnostic
    fn register_config_root(
        &mut self,
        abs_cfg: &AbstractConfig,
        ast_id: AstId,
        scope_type: ScopeType,
        env: &RegistrationEnv,
    ) -> TaggedId<ImplId, ConfigRootTag> {
        debug_assert!(
            matches!(
                abs_cfg.lookup_pat,
                ScopeLookupPattern::NamespaceOnly
                    | ScopeLookupPattern::OnlyVar
                    | ScopeLookupPattern::OnlyNest
            ),
            "Either config of `abs_cfg` was done wrong or a core language change did not update this assertion.\nExpected `ScopeLookupPattern::NoRestrictions/OnlyVar`, found {:?}",
            abs_cfg.lookup_pat
        );
        debug_assert!(matches!(abs_cfg.kind, AbstractConfigKind::Root(_, _)));
        debug_assert!(matches!(scope_type, ScopeType::Complex));

        // Pushing the scope loads all symbols needed by override
        _ = self.compiler.push_scope(scope_type, env.current_mod);

        let impl_id = ImplId::new(self.compiler.impls.len() as u32);
        let tagged = impl_id.into_tagged::<ConfigRootTag>();
        let cfg_root_id = ConfigRootId::new(self.compiler.cfgs.len() as u32);

        // If an original exists, get the key so that it can be reported, otherwise insert it. This
        // is to avoid inserting first and overwriting the last symbol since ergonomically, it
        // probably makes more sense to keep the original for scope searching to fall-back to.

        let common = ConfigRootCommon::new(tagged, cfg_root_id, abs_cfg.lookup_pat, Vec::new());

        let AbstractConfigKind::Root(_, abs_kind) = &abs_cfg.kind else {
            unreachable!()
        };

        // NOTE: Subject to change
        let kind = match abs_kind {
            ConfigRootMetadataKind::Complex => ConfigRootKind::Complex,
            ConfigRootMetadataKind::Override => ConfigRootKind::Override,
        };

        // Purposefully not allocating capacity because this stage is just instantiating, with no
        // promise that the present vec will be appended or moved in any form.
        let cfg_root = ConfigRoot::new(common, None, Vec::new(), kind);

        let impl_hir = ImplHir::new(
            impl_id,
            ImplHirKind::Config(cfg_root_id),
            scope_type,
            Some(ast_id),
        );

        self.compiler.cfgs.push(cfg_root);
        self.compiler.impls.push(impl_hir);
        tagged
    }

    /// Attaches ast_id to the name_id of it's ast structure.
    /// Gives it a unique symbol id and attaches the ast id to it.
    /// Gives the typedef an id attached to `Unknown` which is to be resolved later
    /// Registers the unfinished representation with it's symbol id so that it can still be
    /// referenced
    fn register_typedef(
        &mut self,
        abs_typedef: &AbstractTypeDef,
        ast_id: AstId,
        scope_type: ScopeType,
        env: &RegistrationEnv,
    ) -> TaggedId<SymbolId, TypeDefTag> {
        // Why was this message put here???
        // This will all likely fail eventually
        let scope_id = self.compiler.push_scope(scope_type, env.current_mod);
        let sym_id = SymbolId::new(self.compiler.syms.len() as u32);
        let tagged = sym_id.into_tagged::<TypeDefTag>();

        let table = &mut self.compiler.get_scope_mut(scope_id).scope.table;

        table.ast_to_sym.insert(ast_id, sym_id);

        let orig_sym_opt = if let Some(original) = table.interned_to_sym.get(&abs_typedef.name_id) {
            Some(*original)
        } else {
            table.interned_to_sym.insert(abs_typedef.name_id, sym_id);
            None
        };

        // The actual typedefs position to store inside it's symbol
        let type_def_type_id = TypeId::new(self.compiler.types.len() as u32);

        // The id of the spot where the unknown type is placed, for the typedef
        // May or may not be able to use the reserved Unknown spot
        let inner_type_id = TypeId::new((self.compiler.types.len() + 1) as u32);

        let type_def = TypeDef::new(
            tagged,
            abs_typedef.name_id,
            abs_typedef.name_span,
            inner_type_id,
        );

        let symbol = Symbol::new(
            abs_typedef.name_id,
            sym_id,
            Some(ast_id),
            SymbolOrigin::Module(env.current_mod),
            abs_typedef.is_priv,
            None,
            scope_type,
            SymbolKind::Type(type_def_type_id),
        );

        self.compiler.syms.push(symbol);

        let type_def_info = TypeInfo::new(Type::TypeDef(type_def), env.current_mod);
        self.compiler.types.push(type_def_info);

        // Yes, ty and type should probably be consistent in some form name-wise.
        let inner_ty_info = TypeInfo::new(Type::Unknown, env.current_mod);
        self.compiler.types.push(inner_ty_info);

        if let Some(orig_sym_id) = orig_sym_opt {
            self.report_duplicate(orig_sym_id, sym_id, env);
        }
        tagged
    }

    fn register_struct(
        &mut self,
        abs_struct: &AbstractStruct,
        ast_id: AstId,
        scope_type: ScopeType,
        env: &RegistrationEnv,
    ) -> TaggedId<SymbolId, StructTag> {
        let sym_id = SymbolId::new(self.compiler.syms.len() as u32);
        let tagged = sym_id.into_tagged::<StructTag>();

        let scope_id = self.compiler.push_scope(scope_type, env.current_mod);
        let table = &mut self.compiler.get_scope_mut(scope_id).scope.table;

        table.ast_to_sym.insert(ast_id, sym_id);

        let orig_sym_opt = if let Some(original) = table.interned_to_sym.get(&abs_struct.name_id) {
            Some(*original)
        } else {
            table.interned_to_sym.insert(abs_struct.name_id, sym_id);
            None
        };

        if !abs_struct.is_priv {
            let module = &mut self.compiler.mods[env.current_mod];
            module.exports.push(sym_id);
        }

        let type_id = TypeId::new(self.compiler.types.len() as u32);
        let struct_def = StructDef::new(tagged, abs_struct.name_span, Vec::new());

        let symbol = Symbol::new(
            abs_struct.name_id,
            sym_id,
            Some(ast_id),
            SymbolOrigin::Module(env.current_mod),
            abs_struct.is_priv,
            None,
            scope_type,
            SymbolKind::Type(type_id),
        );

        self.compiler.syms.push(symbol);

        let ty_info = TypeInfo::new(Type::Struct(struct_def), env.current_mod);
        self.compiler.types.push(ty_info);

        if let Some(orig_sym_id) = orig_sym_opt {
            self.report_duplicate(orig_sym_id, sym_id, env);
        }
        tagged
    }

    fn register_enum(
        &mut self,
        abs_enum: &AbstractEnum,
        ast_id: AstId,
        scope_type: ScopeType,
        env: &RegistrationEnv,
    ) -> TaggedId<SymbolId, EnumTag> {
        let scope_id = self.compiler.push_scope(scope_type, env.current_mod);
        let sym_id = SymbolId::new(self.compiler.syms.len() as u32);
        let tagged = sym_id.into_tagged::<EnumTag>();
        let type_id = TypeId::new(self.compiler.types.len() as u32);

        let table = &mut self.compiler.get_scope_mut(scope_id).scope.table;

        table.ast_to_sym.insert(ast_id, sym_id);

        let orig_sym_opt = if let Some(original) = table.interned_to_sym.get(&abs_enum.name_id) {
            Some(*original)
        } else {
            table.interned_to_sym.insert(abs_enum.name_id, sym_id);
            None
        };

        if !abs_enum.is_priv {
            let module = &mut self.compiler.mods[env.current_mod];
            module.exports.push(sym_id);
        }

        let enum_def = EnumDef::new(tagged, abs_enum.name_span, Vec::new());

        let symbol = Symbol::new(
            abs_enum.name_id,
            sym_id,
            Some(ast_id),
            SymbolOrigin::Module(env.current_mod),
            abs_enum.is_priv,
            None,
            scope_type,
            SymbolKind::Type(type_id),
        );

        self.compiler.syms.push(symbol);

        let ty_info = TypeInfo::new(Type::Enum(enum_def), env.current_mod);
        self.compiler.types.push(ty_info);

        if let Some(orig_sym_id) = orig_sym_opt {
            self.report_duplicate(orig_sym_id, sym_id, env);
        }
        tagged
    }

    fn register_alias(
        &mut self,
        abs_alias: &AbstractAlias,
        ast_id: AstId,
        scope_type: ScopeType,
        env: &RegistrationEnv,
    ) -> TaggedId<SymbolId, AliasTag> {
        let scope_id = self.compiler.push_scope(scope_type, env.current_mod);
        let sym_id = SymbolId::new(self.compiler.syms.len() as u32);
        let tagged = sym_id.into_tagged::<AliasTag>();
        let type_id = TypeId::new(self.compiler.types.len() as u32);

        let table = &mut self.compiler.get_scope_mut(scope_id).scope.table;

        table.ast_to_sym.insert(ast_id, sym_id);
        let orig_sym_opt = if let Some(original) = table.interned_to_sym.get(&abs_alias.name_id) {
            Some(*original)
        } else {
            table.interned_to_sym.insert(abs_alias.name_id, sym_id);
            None
        };

        if !abs_alias.is_priv {
            let module = &mut self.compiler.mods[env.current_mod];
            module.exports.push(sym_id);
        }

        // Making local scopes in this way because sections do not emergently allow for
        // parent hierarchies.
        let local_scope_id = ScopeId::new(self.compiler.scopes.len() as u16);
        let local_scope = Scope::new(local_scope_id, ScopeType::Local, false, None);

        self.compiler
            .scopes
            .push(ScopeInfo::new(local_scope, Some(sym_id), env.current_mod));

        let current_mod = &mut self.compiler.mods[env.current_mod];
        current_mod.scopes.push(local_scope_id);

        // Ok ok
        let alias_def = AliasDef::new(
            tagged,
            abs_alias.name_span,
            Vec::new(),
            Vec::new(),
            local_scope_id,
        );

        let symbol = Symbol::new(
            abs_alias.name_id,
            sym_id,
            Some(ast_id),
            SymbolOrigin::Module(env.current_mod),
            abs_alias.is_priv,
            None,
            scope_type,
            SymbolKind::Type(type_id),
        );

        self.compiler.syms.push(symbol);

        let ty_info = TypeInfo::new(Type::Alias(alias_def), env.current_mod);
        self.compiler.types.push(ty_info);

        if let Some(orig_sym_id) = orig_sym_opt {
            self.report_duplicate(orig_sym_id, sym_id, env);
        }
        tagged
    }

    /// Pushes neutral scope if needed, exports variable if public, then stores it with the state
    /// `ReservedTypeSlot` so that it can reserve a type slot without making an expression this
    /// early on, which would complicate the process.
    fn register_var(
        &mut self,
        abs_var: &AbstractVar,
        ast_id: AstId,
        scope_type: ScopeType,
        env: &RegistrationEnv,
    ) -> TaggedId<SymbolId, VarTag> {
        let sym_id = SymbolId::new(self.compiler.syms.len() as u32);
        let tagged = sym_id.into_tagged::<VarTag>();
        let scope_id = self.compiler.push_scope(scope_type, env.current_mod);
        let table = &mut self.compiler.get_scope_mut(scope_id).scope.table;

        table.ast_to_sym.insert(ast_id, sym_id);
        let orig_sym_opt = if let Some(original) = table.interned_to_sym.get(&abs_var.name_id) {
            Some(*original)
        } else {
            table.interned_to_sym.insert(abs_var.name_id, sym_id);
            None
        };

        if !abs_var.is_priv {
            let module = &mut self.compiler.mods[env.current_mod];
            module.exports.push(sym_id);
        }

        let type_id = TypeId::new(self.compiler.types.len() as u32);
        let ty_info = TypeInfo::new(Type::Unknown, env.current_mod);

        let var_id = VariableId::new(self.compiler.variables.len() as u32);

        // TypeId is stored here so that the slot is reserved for anything that may need to refer
        // to it's type before it's actually declared
        let var = VarDef::new(
            tagged,
            abs_var.name_id,
            VariableMetadata::User(abs_var.name_span),
            VariableState::ReservedTypeSlot(type_id),
        );

        // No information that this is a variable other than the fact that AstId -> SymbolId
        let symbol = Symbol::new(
            abs_var.name_id,
            sym_id,
            Some(ast_id),
            SymbolOrigin::Module(env.current_mod),
            abs_var.is_priv,
            None,
            scope_type,
            // Will be SymbolKind::Defer
            SymbolKind::Variable(var_id),
        );

        self.compiler.syms.push(symbol);
        self.compiler.types.push(ty_info);
        self.compiler.variables.push(var);

        if let Some(orig_sym_id) = orig_sym_opt {
            self.report_duplicate(orig_sym_id, sym_id, env);
        }
        tagged
    }

    // Cannot check for this since the type is not known
    /// Forms and stores diagnostic, given an original symbol which has the same identifier as an
    /// existing one
    fn report_duplicate(
        &mut self,
        orig_sym_id: SymbolId,
        dup_sym_id: SymbolId,
        env: &RegistrationEnv,
    ) {
        //NOTE: Suspicious
        let orig_sym = &self.compiler.syms[orig_sym_id];
        let orig_ast_id = orig_sym.ast_id.expect("Core should not be resolved");

        let dup_ast_id = self.compiler.syms[dup_sym_id]
            .ast_id
            .expect("Core should not be resolved");

        let dup_name = self.interner.search(orig_sym.name_id);
        let scope_type = orig_sym.scope_origin;

        let orig_span = env.ast_info.get_decl(orig_ast_id).span();
        let dup_span = env.ast_info.get_decl(dup_ast_id).span();

        let core_msg = format!(
            "Duplicate identifier `{dup_name}` in section `{}`",
            &scope_type
        );

        let src_diag = SourceDiagnostic::builder(
            ErrorCode::ScopeErr.into(),
            DiagnosticLevel::Error,
            core_msg,
            env.region.path_id,
        )
        .add_annotation(
            orig_span,
            AnnotationKind::Secondary,
            format!("`{dup_name}` first seen here").into(),
        )
        .add_annotation(dup_span, AnnotationKind::Primary, None)
        .build();

        self.summary.push_diag(src_diag);
    }
}

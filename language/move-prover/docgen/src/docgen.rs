// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use log::{debug, info, warn};

use codespan::{ByteIndex, Span};
use itertools::Itertools;
use num::BigUint;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use spec_lang::{
    ast::{ModuleName, SpecBlockInfo, SpecBlockTarget},
    code_writer::CodeWriter,
    emit, emitln,
    env::{
        FunctionEnv, GlobalEnv, Loc, ModuleEnv, ModuleId, Parameter, StructEnv, TypeConstraint,
        TypeParameter,
    },
    symbol::Symbol,
    ty::TypeDisplayContext,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    rc::Rc,
};

const KEYWORDS: &[&str] = &[
    "abort",
    "acquires",
    "as",
    "break",
    "continue",
    "copy",
    "copyable",
    "else",
    "false",
    "if",
    "invariant",
    "let",
    "loop",
    "module",
    "move",
    "native",
    "public",
    "resource",
    "return",
    "spec",
    "struct",
    "true",
    "use",
    "while",
    "fun",
    "script",
];

const WEAK_KEYWORDS: &[&str] = &[
    "apply",
    "assert",
    "assume",
    "decreases",
    "aborts_if",
    "ensures",
    "requires",
    "pack",
    "unpack",
    "update",
    "to",
    "schema",
    "include",
    "old",
    "except",
    "global",
    "mut",
    "define",
];

/// Options passed into the documentation generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct DocgenOptions {
    /// The level where we start sectioning. Often markdown sections are rendered with
    /// unnecessary large section fonts, setting this value high reduces the size.
    pub section_level_start: usize,
    /// Whether to include private functions in the generated docs.
    pub include_private_fun: bool,
    /// Whether to include specifications in the generated docs.
    pub include_specs: bool,
    /// Whether to put specifications in the same section as a declaration or put them all
    /// into an independent section.
    pub specs_inlined: bool,
    /// Whether to include Move implementations.
    pub include_impl: bool,
    /// Max depth to which sections are displayed in table-of-contents.
    pub toc_depth: usize,
    /// Whether to use collapsed sections (<details>) for impl and specs
    pub collapsed_sections: bool,
    /// In which directory to store output.
    pub output_directory: String,
    /// In which directories to look for references.
    pub doc_path: Vec<String>,
}

impl Default for DocgenOptions {
    fn default() -> Self {
        Self {
            section_level_start: 1,
            include_private_fun: false,
            include_specs: true,
            specs_inlined: true,
            include_impl: true,
            toc_depth: 3,
            collapsed_sections: true,
            output_directory: "doc".to_string(),
            doc_path: vec!["doc".to_string()],
        }
    }
}

/// The documentation generator.
pub struct Docgen<'env> {
    options: &'env DocgenOptions,
    env: &'env GlobalEnv,
    /// Mapping from module id to the set of schemas defined in this module.
    /// We currently do not have this information in the environment.
    declared_schemas: BTreeMap<ModuleId, BTreeSet<Symbol>>,
    /// Map from module id to generated documentation.
    output: BTreeMap<ModuleId, String>,
    /// Map from module id to the file (relative to output dir) where module is documented.
    ref_map: BTreeMap<ModuleId, String>,
    /// Current code writer.
    writer: CodeWriter,
    /// Current module.
    current_module: Option<ModuleEnv<'env>>,
    /// A mapping from location to spec item defined at this location.
    loc_to_spec_item_map: BTreeMap<Loc, Symbol>,
    /// A table-of-contents list.
    toc: RefCell<Vec<(usize, TocEntry)>>,
    /// The current section next
    section_nest: RefCell<usize>,
}

/// A table-of-contents entry.
#[derive(Debug, Default, Clone)]
struct TocEntry {
    label: String,
    title: String,
}

/// A map from spec block targets to associated spec blocks.
type SpecBlockMap<'a> = BTreeMap<SpecBlockTarget, Vec<&'a SpecBlockInfo>>;

impl<'env> Docgen<'env> {
    /// Creates a new documentation generator.
    pub fn new(env: &'env GlobalEnv, options: &'env DocgenOptions) -> Self {
        Self {
            options,
            env,
            declared_schemas: Default::default(),
            output: Default::default(),
            ref_map: Default::default(),
            writer: CodeWriter::new(env.unknown_loc()),
            current_module: None,
            loc_to_spec_item_map: Default::default(),
            toc: RefCell::new(Default::default()),
            section_nest: RefCell::new(0),
        }
    }

    /// Returns the result of documentation generation, a vector of pairs of filenames
    /// and content.
    pub fn into_result(mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.output)
            .into_iter()
            .map(|(id, content)| {
                let fname = self.ref_map.remove(&id).expect("file");
                let mut path = PathBuf::from(&self.options.output_directory);
                path.push(fname);
                (path.to_string_lossy().to_string(), content)
            })
            .collect_vec()
    }

    /// Generate documentation for all modules in the environment which are not in the dependency
    /// set.
    pub fn gen(&mut self) {
        self.compute_declared_schemas();
        self.compute_file_map();
        for m in self.env.get_modules() {
            if !m.is_dependency() {
                self.gen_module(&m);
            }
        }
    }

    /// Compute the schemas declared in all modules. This information is currently not directly
    /// in the environment, but can be derived from it.
    fn compute_declared_schemas(&mut self) {
        for module_env in self.env.get_modules() {
            let mut schemas = BTreeSet::new();
            for block in module_env.get_spec_block_infos() {
                if let SpecBlockTarget::Schema(_, id, _) = &block.target {
                    schemas.insert(id.symbol());
                }
            }
            self.declared_schemas.insert(module_env.get_id(), schemas);
        }
    }

    /// Computes file locations for all modules in the environment, so they are available
    /// to generate reference links.
    fn compute_file_map(&mut self) {
        for module_env in self.env.get_modules() {
            let output_path = PathBuf::from(&self.options.output_directory);
            let file_name = PathBuf::from(module_env.get_source_path())
                .with_extension("md")
                .file_name()
                .expect("file name")
                .to_os_string();
            let path_opt = if module_env.is_dependency() {
                // Try to locate the file in the provided search path.
                self.options.doc_path.iter().find_map(|dir| {
                    let mut path = PathBuf::from(dir);
                    path.push(&file_name);
                    if path.exists() {
                        Some(
                            self.path_relative_to(&path, &output_path)
                                .to_string_lossy()
                                .to_string(),
                        )
                    } else {
                        None
                    }
                })
            } else {
                // We will generate this file in the provided output directory.
                Some(file_name.to_string_lossy().to_string())
            };
            if let Some(path) = path_opt {
                info!(
                    "{} `{}` in file `{}`",
                    if module_env.is_script_module() {
                        "script"
                    } else {
                        "module"
                    },
                    module_env.get_name().display_full(module_env.symbol_pool()),
                    path
                );
                self.ref_map.insert(module_env.get_id(), path);
            }
        }
    }

    /// Make path relative to other path.
    fn path_relative_to(&self, path: &PathBuf, to: &PathBuf) -> PathBuf {
        if path.is_absolute() || to.is_absolute() {
            path.clone()
        } else {
            let mut result = PathBuf::new();
            for _ in to.components() {
                result.push("..");
            }
            result.join(path)
        }
    }

    /// Generates documentation for a module.
    fn gen_module(&mut self, module_env: &ModuleEnv<'env>) {
        // (Re-) initialize state for this module.
        self.writer = CodeWriter::new(self.env.unknown_loc());
        self.toc = RefCell::new(Default::default());
        self.section_nest = RefCell::new(0);
        self.current_module = Some(module_env.clone());
        self.loc_to_spec_item_map.clear();
        for (_, sfun) in module_env.get_spec_funs() {
            self.loc_to_spec_item_map
                .insert(sfun.loc.clone(), sfun.name);
        }
        for (_, svar) in module_env.get_spec_vars() {
            self.loc_to_spec_item_map
                .insert(svar.loc.clone(), svar.name);
        }

        let module_name = format!(
            "{}",
            module_env.get_name().display_full(module_env.symbol_pool())
        );
        if module_env.is_script_module() {
            let file = PathBuf::from(module_env.get_source_path())
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();
            self.section_header(
                &format!("Script `{}`", file),
                &self.label_for_module(module_env),
            );
        } else {
            self.section_header(
                &format!("Module `{}`", module_name),
                &self.label_for_module(module_env),
            );
        }
        self.increment_section_nest();
        // Do TOC header manually so it does not appear in TOC.
        emitln!(
            self.writer,
            "{} Table of Contents",
            self.repeat_str("#", self.options.section_level_start + 2)
        );

        // Create label where we later can insert the TOC
        emitln!(self.writer);
        let toc_label = self.writer.create_label();
        emitln!(self.writer);

        self.doc_text(module_env, module_env.get_doc());

        let spec_block_map = self.organize_spec_blocks(module_env);

        if self.options.specs_inlined {
            self.gen_spec_blocks(module_env, "", &SpecBlockTarget::Module, &spec_block_map);
        }

        if !module_env.get_structs().count() > 0 {
            for s in module_env
                .get_structs()
                .sorted_by(|a, b| Ord::cmp(&a.get_loc(), &b.get_loc()))
            {
                self.gen_struct(&spec_block_map, &s);
            }
        }

        let funs = module_env
            .get_functions()
            .filter(|f| self.options.include_private_fun || f.is_public())
            .sorted_by(|a, b| Ord::cmp(&a.get_loc(), &b.get_loc()))
            .collect_vec();
        if !funs.is_empty() {
            for f in funs {
                self.gen_function(&spec_block_map, &f);
            }
        }

        if !self.options.specs_inlined {
            self.gen_spec_section(module_env, &spec_block_map);
        }
        self.decrement_section_nest();

        // Generate table of contents. We put this into a separate code writer which we then
        // merge into the main one.
        let writer = std::mem::replace(&mut self.writer, CodeWriter::new(self.env.unknown_loc()));
        {
            let mut level = 0;
            for (nest, entry) in self
                .toc
                .borrow()
                .iter()
                .filter(|(n, _)| *n > 0 && *n <= self.options.toc_depth)
            {
                let n = *nest - 1;
                while level < n {
                    self.begin_items();
                    self.writer.indent();
                    level += 1;
                }
                while level > n {
                    self.end_items();
                    self.writer.unindent();
                    level -= 1;
                }
                self.item_text(None, &format!("[{}](#{})", entry.title, entry.label));
            }
            while level > 0 {
                self.end_items();
                self.writer.unindent();
                level -= 1;
            }
            self.writer
                .process_result(|s| writer.insert_at_label(toc_label, s));
        }

        // Store result in output map.
        writer.process_result(|s| self.output.insert(module_env.get_id(), s.to_owned()));
    }

    /// Generates documentation for a struct.
    fn gen_struct(&self, spec_block_map: &SpecBlockMap<'_>, struct_env: &StructEnv<'_>) {
        let name = struct_env.get_name();
        self.section_header(
            &format!("Struct `{}`", self.name_string(name)),
            &self.label_for_module_item(&struct_env.module_env, name),
        );
        self.increment_section_nest();
        self.doc_text(&struct_env.module_env, struct_env.get_doc());
        self.code_block(
            &struct_env.module_env,
            &self.struct_header_display(struct_env),
        );

        if self.options.include_impl || (self.options.include_specs && self.options.specs_inlined) {
            // Include field documentation if either impls or specs are present and inlined,
            // because they are used by both.
            self.begin_collapsed("Fields");
            self.gen_struct_fields(struct_env);
            self.end_collapsed();
        }

        if self.options.specs_inlined {
            self.gen_spec_blocks(
                &struct_env.module_env,
                "Specification",
                &SpecBlockTarget::Struct(struct_env.module_env.get_id(), struct_env.get_id()),
                spec_block_map,
            );
        }
        self.decrement_section_nest();
    }

    /// Generates code signature for a struct.
    fn struct_header_display(&self, struct_env: &StructEnv<'_>) -> String {
        let name = self.name_string(struct_env.get_name());
        let kind = if struct_env.is_resource() {
            "resource struct"
        } else {
            "struct"
        };
        format!(
            "{} {}{}",
            kind,
            name,
            self.type_parameter_list_display(&struct_env.get_named_type_parameters()),
        )
    }

    fn gen_struct_fields(&self, struct_env: &StructEnv<'_>) {
        let tctx = self.type_display_context_for_struct(struct_env);
        self.begin_definitions();
        for field in struct_env.get_fields() {
            self.definition_text(
                Some(&struct_env.module_env),
                &format!(
                    "`{}: {}`",
                    self.name_string(field.get_name()),
                    field.get_type().display(&tctx)
                ),
                field.get_doc(),
            );
        }
        self.end_definitions();
    }

    /// Generates documentation for a function.
    fn gen_function(&self, spec_block_map: &SpecBlockMap<'_>, func_env: &FunctionEnv<'_>) {
        let name = func_env.get_name();
        self.section_header(
            &format!("Function `{}`", self.name_string(name)),
            &self.label_for_module_item(&func_env.module_env, name),
        );
        self.increment_section_nest();
        self.doc_text(&func_env.module_env, func_env.get_doc());
        let sig = self.function_header_display(func_env);
        self.code_block(&func_env.module_env, &sig);
        if self.options.specs_inlined {
            self.gen_spec_blocks(
                &func_env.module_env,
                "Specification",
                &SpecBlockTarget::Function(func_env.module_env.get_id(), func_env.get_id()),
                spec_block_map,
            )
        }
        if self.options.include_impl {
            self.begin_collapsed("Implementation");
            self.code_block(
                &func_env.module_env,
                &self.get_source_with_indent(&func_env.get_loc()),
            );
            self.end_collapsed();
        }
        self.decrement_section_nest();
    }

    /// Generates documentation for a function signature.
    fn function_header_display(&self, func_env: &FunctionEnv<'_>) -> String {
        let name = self.name_string(func_env.get_name());
        let visibility = if func_env.is_public() { "public " } else { "" };
        let tctx = &self.type_display_context_for_fun(&func_env);
        let params = func_env
            .get_parameters()
            .iter()
            .map(|Parameter(name, ty)| format!("{}: {}", self.name_string(*name), ty.display(tctx)))
            .join(", ");
        let return_types = func_env.get_return_types();
        let return_str = match return_types.len() {
            0 => "".to_owned(),
            1 => format!(": {}", return_types[0].display(tctx)),
            _ => format!(
                ": ({})",
                return_types.iter().map(|ty| ty.display(tctx)).join(", ")
            ),
        };
        format!(
            "{}fun {}{}({}){}",
            visibility,
            name,
            self.type_parameter_list_display(&func_env.get_named_type_parameters()),
            params,
            return_str
        )
    }

    /// Generates documentation for a series of spec blocks associated with spec block target.
    fn gen_spec_blocks(
        &self,
        module_env: &ModuleEnv<'_>,
        title: &str,
        target: &SpecBlockTarget,
        spec_block_map: &SpecBlockMap,
    ) {
        let no_blocks = &vec![];
        let blocks = spec_block_map.get(target).unwrap_or(no_blocks);
        if blocks.is_empty() || !self.options.include_specs {
            return;
        }
        if !title.is_empty() {
            self.begin_collapsed(title);
        }
        for block in blocks {
            self.doc_text(module_env, self.env.get_doc(&block.loc));
            let mut in_code = false;
            let (is_schema, schema_header) =
                if let SpecBlockTarget::Schema(_, sid, type_params) = &block.target {
                    self.label(&format!(
                        "{}_{}",
                        self.label_for_module(module_env),
                        self.name_string(sid.symbol())
                    ));
                    (
                        true,
                        format!(
                            "schema {}{} {{",
                            self.name_string(sid.symbol()),
                            self.type_parameter_list_display(type_params)
                        ),
                    )
                } else {
                    (false, "".to_owned())
                };
            let begin_code = |in_code: &mut bool| {
                if !*in_code {
                    self.begin_code();
                    if is_schema {
                        self.code_text(module_env, &schema_header);
                        self.writer.indent();
                    }
                    *in_code = true;
                }
            };
            let end_code = |in_code: &mut bool| {
                if *in_code {
                    if is_schema {
                        self.writer.unindent();
                        self.code_text(module_env, "}");
                    }
                    self.end_code();
                    *in_code = false;
                }
            };
            for loc in &block.member_locs {
                let doc = self.env.get_doc(loc);
                if !doc.is_empty() {
                    end_code(&mut in_code);
                    self.doc_text(module_env, doc);
                }
                // Inject label for spec item definition.
                if let Some(item) = self.loc_to_spec_item_map.get(loc) {
                    let label = &format!(
                        "{}_{}",
                        self.label_for_module(module_env),
                        self.name_string(*item)
                    );
                    if in_code {
                        self.label_in_code(label);
                    } else {
                        self.label(label);
                    }
                }
                begin_code(&mut in_code);
                self.code_text(module_env, &self.get_source_with_indent(loc));
            }
            end_code(&mut in_code);
        }
        if !title.is_empty() {
            self.end_collapsed();
        }
    }

    /// Organizes spec blocks in the module such that free items like schemas and module blocks
    /// are associated with the context they appear in.
    fn organize_spec_blocks(&self, module_env: &'env ModuleEnv<'env>) -> SpecBlockMap<'env> {
        let mut result = BTreeMap::new();
        let mut current_target = SpecBlockTarget::Module;
        for block in module_env.get_spec_block_infos() {
            match &block.target {
                SpecBlockTarget::Schema(..) => {}
                SpecBlockTarget::Module if !self.is_single_liner(&block.loc) => {
                    // This is a bit of a hack: if spec module is on a single line,
                    // we consider it as a marker to switch doc context back to module level.
                }
                _ => {
                    // Switch target if it's not a schema or module. Those will be associated with
                    // the last target.
                    current_target = block.target.clone();
                }
            }
            result
                .entry(current_target.clone())
                .or_insert_with(Vec::new)
                .push(block);
        }
        result
    }

    /// Check whether the location contains a single line of source.
    fn is_single_liner(&self, loc: &Loc) -> bool {
        self.env
            .get_source(loc)
            .map(|s| !s.contains('\n'))
            .unwrap_or(false)
    }

    /// Generates standalone spec section. This is used if `options.specs_inlined` is false.
    fn gen_spec_section(&self, module_env: &ModuleEnv<'_>, spec_block_map: &SpecBlockMap<'_>) {
        if spec_block_map.is_empty() || !self.options.include_specs {
            return;
        }
        let section_label = self.label_for_module_item_str(module_env, "Specification");
        self.section_header("Specification", &section_label);
        self.increment_section_nest();
        self.gen_spec_blocks(module_env, "", &SpecBlockTarget::Module, spec_block_map);
        for struct_env in module_env
            .get_structs()
            .sorted_by(|a, b| Ord::cmp(&a.get_loc(), &b.get_loc()))
        {
            let target =
                SpecBlockTarget::Struct(struct_env.module_env.get_id(), struct_env.get_id());
            if spec_block_map.contains_key(&target) {
                let name = self.name_string(struct_env.get_name());
                self.section_header(
                    &format!("Struct `{}`", name),
                    &format!("{}_{}", section_label, name),
                );
                self.code_block(module_env, &self.struct_header_display(&struct_env));
                self.gen_struct_fields(&struct_env);
                self.gen_spec_blocks(module_env, "", &target, spec_block_map);
            }
        }
        for func_env in module_env
            .get_functions()
            .sorted_by(|a, b| Ord::cmp(&a.get_loc(), &b.get_loc()))
        {
            let target = SpecBlockTarget::Function(func_env.module_env.get_id(), func_env.get_id());
            if spec_block_map.contains_key(&target) {
                let name = self.name_string(func_env.get_name());
                self.section_header(
                    &format!("Function `{}`", name),
                    &format!("{}_{}", section_label, name),
                );
                self.code_block(
                    &func_env.module_env,
                    &self.function_header_display(&func_env),
                );
                self.gen_spec_blocks(module_env, "", &target, spec_block_map);
            }
        }
        self.decrement_section_nest();
    }

    // ============================================================================================
    // Helpers

    /// Returns a string for a name symbol.
    fn name_string(&self, name: Symbol) -> Rc<String> {
        self.env.symbol_pool().string(name)
    }

    /// Creates a type display context for a function.
    fn type_display_context_for_fun(&self, func_env: &FunctionEnv<'_>) -> TypeDisplayContext<'_> {
        let type_param_names = Some(
            func_env
                .get_named_type_parameters()
                .iter()
                .map(|TypeParameter(name, _)| *name)
                .collect_vec(),
        );
        TypeDisplayContext::WithEnv {
            env: self.env,
            type_param_names,
        }
    }

    /// Creates a type display context for a struct.
    fn type_display_context_for_struct(
        &self,
        struct_env: &StructEnv<'_>,
    ) -> TypeDisplayContext<'_> {
        let type_param_names = Some(
            struct_env
                .get_named_type_parameters()
                .iter()
                .map(|TypeParameter(name, _)| *name)
                .collect_vec(),
        );
        TypeDisplayContext::WithEnv {
            env: self.env,
            type_param_names,
        }
    }

    /// Increments section nest.
    fn increment_section_nest(&self) {
        *self.section_nest.borrow_mut() += 1;
    }

    /// Decrements section nest, committing sub-sections to the table-of-contents map.
    fn decrement_section_nest(&self) {
        *self.section_nest.borrow_mut() -= 1;
    }

    /// Creates a new section header and inserts a table-of-contents entry into the generator.
    fn section_header(&self, s: &str, label: &str) {
        let level = *self.section_nest.borrow();
        if !label.is_empty() {
            self.label(label);
            let entry = TocEntry {
                title: s.to_owned(),
                label: label.to_string(),
            };
            self.toc.borrow_mut().push((level, entry));
        }
        emitln!(
            self.writer,
            "{} {}",
            self.repeat_str("#", self.options.section_level_start + level),
            s,
        );
        emitln!(self.writer);
    }

    /// Generate label.
    fn label(&self, label: &str) {
        emitln!(self.writer);
        emitln!(self.writer, "<a name=\"{}\"></a>", label);
        emitln!(self.writer);
    }

    /// Generate label in code, without empty lines.
    fn label_in_code(&self, label: &str) {
        emitln!(self.writer, "<a name=\"{}\"></a>", label);
    }

    /// Begins a collapsed section.
    fn begin_collapsed(&self, summary: &str) {
        if self.options.collapsed_sections {
            emitln!(self.writer);
            emitln!(self.writer, "<details>");
            emitln!(self.writer, "<summary>{}</summary>", summary);
            emitln!(self.writer);
        } else {
            emitln!(self.writer);
            emitln!(self.writer, "##### {}", summary);
            emitln!(self.writer);
        }
    }

    /// Ends a collapsed section.
    fn end_collapsed(&self) {
        if self.options.collapsed_sections {
            emitln!(self.writer);
            emitln!(self.writer, "</details>");
        }
    }

    /// Outputs documentation text.
    fn doc_text(&self, module_env: &ModuleEnv<'_>, text: &str) {
        for line in self.decorate_text(module_env, text).lines() {
            let line = line.trim();
            if line.starts_with('#') {
                let mut i = 1;
                while line[i..].starts_with('#') {
                    i += 1;
                    self.increment_section_nest();
                }
                let header = line[i..].trim_start();
                self.section_header(
                    header,
                    &self.label_for_module_item_str(module_env, &self.make_label_from_str(header)),
                );
                while i > 1 {
                    self.decrement_section_nest();
                    i -= 1;
                }
            } else {
                emitln!(self.writer, line)
            }
        }
        // Always be sure to have an empty line at the end of block.
        emitln!(self.writer);
    }

    /// Makes a label from a string.
    fn make_label_from_str(&self, s: &str) -> String {
        format!("@{}", s.replace(' ', "_"))
    }

    /// Decorates documentation text, identifying code fragments and decorating them
    /// as code.
    fn decorate_text(&self, module_env: &ModuleEnv<'_>, text: &str) -> String {
        static REX: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)`[^`]+`").unwrap());
        let mut r = String::new();
        let mut at = 0;
        while let Some(m) = REX.find(&text[at..]) {
            // If this is ``` skip it.
            let end = at + m.end();
            if text[end..].starts_with('`') {
                r += &text[at..end];
                at = end;
                continue;
            }
            r += &text[at..at + m.start()];
            // If the current text does not start on a newline, we need to add one.
            // Some markdown processors won't recognize a <code> if its not on a new
            // line. However, if we insert a blank line, we get a paragraph break, so we
            // need to avoid this.
            if !r.trim_end_matches(' ').ends_with('\n') {
                r += "\n";
            }
            r += &format!(
                "<code>{}</code>",
                &self.decorate_code(module_env, &m.as_str()[1..&m.as_str().len() - 1])
            );
            at += m.end();
        }
        r += &text[at..];
        r
    }

    /// Begins a code block. This uses html, not markdown code blocks, so we are able to
    /// insert style and links into the code.
    fn begin_code(&self) {
        emitln!(self.writer);
        // If we newline after <pre><code>, an empty line will be created. So we don't.
        // This, however, creates some ugliness with indented code.
        emit!(self.writer, "<pre><code>");
    }

    /// Ends a code block.
    fn end_code(&self) {
        emitln!(self.writer, "</code></pre>\n");
        // Always be sure to have an empty line at the end of block.
        emitln!(self.writer);
    }

    /// Outputs decorated code text in context of a module.
    fn code_text(&self, module_env: &ModuleEnv<'_>, code: &str) {
        emitln!(self.writer, &self.decorate_code(module_env, code));
    }

    /// Decorates a code fragment, for use in an html block. Replaces < and >, bolds keywords and
    /// tries to resolve and cross-link references.
    fn decorate_code(&self, module_env: &ModuleEnv<'_>, code: &str) -> String {
        static REX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?P<ident>(\b\w+\b\s*::\s*)*\b\w+\b)(?P<call>\s*[(<])?|(?P<lt><)|(?P<gt>>)",
            )
            .unwrap()
        });
        let mut r = String::new();
        let mut at = 0;
        while let Some(cap) = REX.captures(&code[at..]) {
            let replacement = {
                if cap.name("lt").is_some() {
                    "&lt;".to_owned()
                } else if cap.name("gt").is_some() {
                    "&gt;".to_owned()
                } else if let Some(m) = cap.name("ident") {
                    let is_call = cap.name("call").is_some();
                    let s = m.as_str();
                    if KEYWORDS.contains(&s) || WEAK_KEYWORDS.contains(&s) {
                        format!("<b>{}</b>", &code[at + m.start()..at + m.end()])
                    } else if let Some(label) = self.resolve_to_label(module_env, s, is_call) {
                        format!("<a href=\"{}\">{}</a>", label, s)
                    } else {
                        "".to_owned()
                    }
                } else {
                    "".to_owned()
                }
            };
            if replacement.is_empty() {
                r += &code[at..at + cap.get(0).unwrap().end()].replace("<", "&lt;");
            } else {
                r += &code[at..at + cap.get(0).unwrap().start()];
                r += &replacement;
                if let Some(m) = cap.name("call") {
                    // Append the call or generic open we may have also matched to distinguish
                    // a simple name from a function call or generic instantiation. Need to
                    // replace the `<` as well.
                    r += &m.as_str().replace("<", "&lt;");
                }
            }
            at += cap.get(0).unwrap().end();
        }
        r += &code[at..];
        r
    }

    /// Resolve a string of the form `ident`, `ident::ident`, or `0xN::ident::ident` into
    /// the label for the declaration inside of this documentation. This uses a
    /// heuristic and may not work in all cases or produce wrong results (for instance, it
    /// ignores aliases). To improve on this, we would need best direct support by the compiler.
    fn resolve_to_label(
        &self,
        module_env: &ModuleEnv<'_>,
        s: &str,
        is_followed_by_open: bool,
    ) -> Option<String> {
        let parts_data: Vec<&str> = s.splitn(3, "::").collect();
        let mut parts = parts_data.as_slice();
        let module_opt = if parts[0].starts_with("0x") {
            if parts.len() == 1 {
                // Cannot resolve.
                return None;
            }
            let addr = BigUint::parse_bytes(&parts[0][2..].as_bytes(), 16)?;
            let mname = ModuleName::new(addr, self.env.symbol_pool().make(parts[1]));
            parts = &parts[2..];
            Some(self.env.find_module(&mname)?)
        } else {
            None
        };
        let try_func_or_struct = |module: &ModuleEnv<'_>, name: Symbol, is_qualified: bool| {
            // Below we only resolve a simple name to a function if it is followed by a ( or <.
            // Otherwise we get too many false positives where names are resolved to functions
            // but are actually fields.
            if module.find_struct(name).is_some()
                || module.find_spec_var(name).is_some()
                || self
                    .declared_schemas
                    .get(&module.get_id())
                    .map(|s| s.contains(&name))
                    .unwrap_or(false)
                || ((is_qualified || is_followed_by_open)
                    && (module.find_function(name).is_some()
                        || module.get_spec_funs_of_name(name).next().is_some()))
            {
                Some(self.ref_for_module_item(&module, name))
            } else {
                None
            }
        };
        let parts_sym = parts
            .iter()
            .map(|p| self.env.symbol_pool().make(p))
            .collect_vec();

        match (module_opt, parts_sym.len()) {
            (Some(module), 0) => Some(self.ref_for_module(&module)),
            (Some(module), 1) => try_func_or_struct(&module, parts_sym[0], true),
            (None, 0) => None,
            (None, 1) => {
                // A simple name. Resolve either to module or to item in current module.
                if let Some(module) = self.env.find_module_by_name(parts_sym[0]) {
                    Some(self.ref_for_module(&module))
                } else {
                    try_func_or_struct(module_env, parts_sym[0], false)
                }
            }
            (None, 2) => {
                // A qualified name, but without the address. This must be an item in a module
                // denoted by the first name.
                let module_opt = if parts[0] == "Self" {
                    Some(module_env.clone())
                } else {
                    self.env.find_module_by_name(parts_sym[0])
                };
                if let Some(module) = module_opt {
                    try_func_or_struct(&module, parts_sym[1], true)
                } else {
                    None
                }
            }
            (_, _) => None,
        }
    }

    /// Return the label for a module.
    fn label_for_module(&self, module_env: &ModuleEnv<'_>) -> String {
        if module_env.is_script_module() {
            "SCRIPT".to_string()
        } else {
            module_env
                .get_name()
                .display_full(self.env.symbol_pool())
                .to_string()
                .replace("::", "_")
        }
    }

    /// Return the reference for a module.
    fn ref_for_module(&self, module_env: &ModuleEnv<'_>) -> String {
        let label = self.label_for_module(module_env);
        if let Some(current) = &self.current_module {
            if current.get_id() == module_env.get_id() {
                return format!("#{}", label);
            }
        }
        let file = self
            .ref_map
            .get(&module_env.get_id())
            .map(|s| s.as_ref())
            .unwrap_or("");
        format!("{}#{}", file, label)
    }

    /// Return the label for an item in a module.
    fn label_for_module_item(&self, module_env: &ModuleEnv<'_>, item: Symbol) -> String {
        self.label_for_module_item_str(module_env, self.name_string(item).as_str())
    }

    /// Return the label for an item in a module.
    fn label_for_module_item_str(&self, module_env: &ModuleEnv<'_>, s: &str) -> String {
        format!("{}_{}", self.label_for_module(module_env), s)
    }

    /// Return the reference for an item in a module.
    fn ref_for_module_item(&self, module_env: &ModuleEnv<'_>, item: Symbol) -> String {
        format!(
            "{}_{}",
            self.ref_for_module(module_env),
            item.display(self.env.symbol_pool())
        )
    }

    /// Shortcut for code_block in a module context.
    fn code_block(&self, module_env: &ModuleEnv<'_>, code: &str) {
        self.begin_code();
        self.code_text(module_env, code);
        self.end_code();
    }

    /// Begin an itemized list.
    fn begin_items(&self) {}

    /// End an itemized list.
    fn end_items(&self) {}

    /// Emit an item. With use the optional module env to do text decoration.
    fn item_text(&self, module_env_opt: Option<&ModuleEnv<'_>>, text: &str) {
        match module_env_opt {
            Some(module_env) => emitln!(self.writer, "-  {}", self.decorate_text(module_env, text)),
            None => emitln!(self.writer, "-  {}", text),
        }
    }

    /// Begin a definition list.
    fn begin_definitions(&self) {
        emitln!(self.writer);
        emitln!(self.writer, "<dl>");
    }

    /// End a definition list.
    fn end_definitions(&self) {
        emitln!(self.writer, "</dl>");
        emitln!(self.writer);
    }

    /// Emit a definition.
    fn definition_text(&self, module_env_opt: Option<&ModuleEnv<'_>>, term: &str, def: &str) {
        let decorate = |s| match module_env_opt {
            Some(m) => self.decorate_text(m, s),
            None => s.to_string(),
        };
        emitln!(self.writer, "<dt>\n{}\n</dt>", decorate(term));
        emitln!(self.writer, "<dd>\n{}\n</dd>", decorate(def));
    }

    /// Display a type parameter.
    fn type_parameter_display(&self, tp: &TypeParameter) -> String {
        format!(
            "{}{}",
            self.name_string(tp.0),
            match tp.1 {
                TypeConstraint::None => "",
                TypeConstraint::Resource => ": resource",
                TypeConstraint::Copyable => ": copyable",
            }
        )
    }

    /// Display a type parameter list.
    fn type_parameter_list_display(&self, tps: &[TypeParameter]) -> String {
        if tps.is_empty() {
            "".to_owned()
        } else {
            format!(
                "<{}>",
                tps.iter()
                    .map(|tp| self.type_parameter_display(tp))
                    .join(", ")
            )
        }
    }

    /// Retrieves source of code fragment with adjusted indentation.
    /// Typically code has the first line unindented because location tracking starts
    /// at the first keyword of the item (e.g. `public fun`), but subsequent lines are then
    /// indented. This uses a heuristic by guessing the indentation from the context.
    fn get_source_with_indent(&self, loc: &Loc) -> String {
        if let Ok(source) = self.env.get_source(loc) {
            // Compute the indentation of this source fragment by looking at some
            // characters preceding it.
            let mut peek_start = loc.span().start().0;
            if peek_start > 60 {
                peek_start -= 60;
            } else {
                peek_start = 0;
            }
            let source_before = self
                .env
                .get_source(&Loc::new(
                    loc.file_id(),
                    Span::new(ByteIndex(peek_start), loc.span().start()),
                ))
                .unwrap_or("");
            let newl_at = source_before.rfind('\n').unwrap_or(0);
            let indent = source_before.len() - newl_at - 1;
            // Remove the indent from all lines.
            source
                .lines()
                .map(|l| {
                    let mut i = 0;
                    while i < indent && i < l.len() && l[i..].starts_with(' ') {
                        i += 1;
                    }
                    &l[i..]
                })
                .join("\n")
        } else {
            "<unknown source>".to_string()
        }
    }

    /// Repeats a string n times.
    fn repeat_str(&self, s: &str, n: usize) -> String {
        (0..n).map(|_| s).collect::<String>()
    }
}

// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use log::{info, warn};

use itertools::Itertools;
use num::BigUint;
use regex::Regex;
use serde::Serialize;
use spec_lang::{
    ast::{ModuleName, SpecBlockInfo, SpecBlockTarget},
    code_writer::CodeWriter,
    emit, emitln,
    env::{FunctionEnv, GlobalEnv, ModuleEnv, Parameter, StructEnv, TypeConstraint, TypeParameter},
    symbol::Symbol,
    ty::TypeDisplayContext,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
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
];

/// Options passed into the documentation generator.
#[derive(Debug, Clone, Serialize)]
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
        }
    }
}

/// The documentation generator.
pub struct Docgen<'env> {
    env: &'env GlobalEnv,
    options: &'env DocgenOptions,
    writer: CodeWriter,
    toc: RefCell<VecDeque<Vec<TocEntry>>>,
}

/// A table-of-contents entry.
#[derive(Debug, Default)]
struct TocEntry {
    label: String,
    title: String,
    sub_entries: Vec<TocEntry>,
}

/// A map from spec block targets to associated spec blocks.
type SpecBlockMap<'a> = BTreeMap<SpecBlockTarget, Vec<&'a SpecBlockInfo>>;

impl<'env> Docgen<'env> {
    /// Creates a new documentation generator.
    pub fn new(env: &'env GlobalEnv, options: &'env DocgenOptions) -> Self {
        let mut toc = VecDeque::new();
        toc.push_back(Vec::new());
        Self {
            env,
            options,
            writer: CodeWriter::new(env.unknown_loc()),
            toc: RefCell::new(toc),
        }
    }

    /// Returns the code writer of this generator, consuming it.
    pub fn into_code_writer(self) -> CodeWriter {
        self.writer
    }

    /// Generate documentation for all modules in the environment which are not in the dependency
    /// set.
    pub fn gen(&mut self) {
        emitln!(
            self.writer,
            "{} Table of Contents",
            self.repeat_str("#", self.options.section_level_start + 1)
        );
        let toc_label = self.writer.create_label();
        for m in self.env.get_modules() {
            if !m.is_in_dependency() {
                self.gen_module(&m);
            }
        }

        // Generate table of contents. We put this into a separate code writer which we then
        // merge into the main one.
        let mut toc_writer = CodeWriter::new(self.env.unknown_loc());
        let writer = std::mem::replace(&mut self.writer, toc_writer);
        let toc = self.toc.borrow();
        assert_eq!(toc.len(), 1, "unbalanced sectioning");
        self.begin_items();
        for entry in toc.front().unwrap() {
            self.gen_toc(entry, 1);
        }
        self.end_items();
        toc_writer = std::mem::replace(&mut self.writer, writer);
        toc_writer.process_result(|s| self.writer.insert_at_label(toc_label, s));
    }

    /// Generates a toc-entry, recursively.
    fn gen_toc(&self, entry: &TocEntry, depth: usize) {
        if depth > self.options.toc_depth {
            return;
        }
        self.item_text(None, &format!("[{}](#{})", entry.title, entry.label));
        if !entry.sub_entries.is_empty() {
            self.writer.indent();
            self.begin_items();
            for sub_entry in &entry.sub_entries {
                self.gen_toc(sub_entry, depth + 1);
            }
            self.end_items();
            self.writer.unindent();
        }
    }

    /// Generates documentation for a module.
    fn gen_module(&self, module_env: &ModuleEnv<'_>) {
        let module_name = format!(
            "{}",
            module_env.get_name().display_full(module_env.symbol_pool())
        );
        self.section_header(
            &format!("Module `{}`", module_name),
            &self.label_for_module(module_env),
        );
        self.increment_section_nest();
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
    }

    /// Generates documentation for a struct.
    fn gen_struct(&self, spec_block_map: &SpecBlockMap<'_>, struct_env: &StructEnv<'_>) {
        let name = self.name_string(struct_env.get_name());
        self.section_header(&format!("Struct `{}`", name), name.as_str());
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
        let name = self.name_string(func_env.get_name());
        self.section_header(&format!("Function `{}`", name), name.as_str());
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
            let source = match self.env.get_source(&func_env.get_loc()) {
                Ok(s) => s,
                Err(_) => "<source unavailable>",
            };
            self.code_block(&func_env.module_env, &self.fix_indent(source));
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
                let member_source = match self.env.get_source(loc) {
                    Ok(s) => s,
                    Err(_) => "<unknown source>",
                };
                begin_code(&mut in_code);
                self.code_text(module_env, member_source);
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
            if !matches!(
                block.target,
                SpecBlockTarget::Schema(..) | SpecBlockTarget::Module
            ) {
                // Switch target if it's not a schema or module. Those will be associated with
                // the last target.
                current_target = block.target.clone();
            }
            result
                .entry(current_target.clone())
                .or_insert_with(Vec::new)
                .push(block);
        }
        result
    }

    /// Generates standalone spec section. This is used if `options.specs_inlined` is false.
    fn gen_spec_section(&self, module_env: &ModuleEnv<'_>, spec_block_map: &SpecBlockMap<'_>) {
        if spec_block_map.is_empty() || !self.options.include_specs {
            return;
        }
        self.section_header("Specification", "Specification");
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
                self.section_header(&format!("Struct `{}`", name), name.as_str());
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
                self.section_header(&format!("Function `{}`", name), name.as_str());
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
        self.toc.borrow_mut().push_back(Vec::new());
    }

    /// Decrements section nest, committing sub-sections to the table-of-contents map.
    fn decrement_section_nest(&self) {
        let mut toc = self.toc.borrow_mut();
        let sub_sections = toc.pop_back().expect("unbalanced sections");
        if !sub_sections.is_empty() {
            let entry = toc
                .back_mut()
                .expect("balanced section nest")
                .last_mut()
                .expect("linear sectioning");
            entry.sub_entries = sub_sections;
        }
    }

    /// Creates a new section header and inserts a table-of-contents entry into the generator.
    fn section_header(&self, s: &str, relative_label: &str) {
        let level = self.toc.borrow().len();
        let parent_label = if level > 1 {
            self.toc.borrow()[level - 2]
                .last()
                .map(|entry| entry.label.to_string())
        } else {
            None
        };
        let qualified_label = if let Some(l) = parent_label {
            format!("{}_{}", l, relative_label)
        } else {
            relative_label.to_string()
        };
        emitln!(self.writer);
        emitln!(self.writer, "<a name=\"{}\"></a>", qualified_label);
        emitln!(self.writer);
        emitln!(
            self.writer,
            "{} {}",
            self.repeat_str(
                "#",
                self.options.section_level_start + self.toc.borrow().len()
            ),
            s,
        );
        emitln!(self.writer);
        let entry = TocEntry {
            title: s.to_owned(),
            label: qualified_label,
            sub_entries: vec![],
        };
        self.toc
            .borrow_mut()
            .back_mut()
            .expect("linear sectioning")
            .push(entry);
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
                // Add current section level
                emitln!(
                    self.writer,
                    "{}{}",
                    self.repeat_str(
                        "#",
                        self.options.section_level_start + self.toc.borrow().len() - 1
                    ),
                    line
                );
            } else {
                emitln!(self.writer, line)
            }
        }
        // Always be sure to have an empty line at the end of block.
        emitln!(self.writer);
    }

    /// Decorates documentation text, identifying code fragments and decorating them
    /// as code.
    fn decorate_text(&self, module_env: &ModuleEnv<'_>, text: &str) -> String {
        let rex = Regex::new(r"(?m)`[^`]+`").unwrap();
        let mut r = String::new();
        let mut at = 0;
        while let Some(m) = rex.find(&text[at..]) {
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
        let rex = Regex::new(
            r"(?P<ident>(\b\w+\b\s*::\s*)*\b\w+\b)(?P<call>\s*[(<])?|(?P<lt><)|(?P<gt>>)",
        )
        .unwrap();
        let mut r = String::new();
        let mut at = 0;
        while let Some(cap) = rex.captures(&code[at..]) {
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
                        format!("<a href=\"#{}\">{}</a>", label, s)
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
        let skip_dependency = |module: ModuleEnv<'env>| -> Option<ModuleEnv<'env>> {
            if module.is_in_dependency() {
                // Don't create references for code not part of this documentation.
                None
            } else {
                Some(module)
            }
        };
        let module_opt = if parts[0].starts_with("0x") {
            if parts.len() == 1 {
                // Cannot resolve.
                return None;
            }
            let addr = BigUint::parse_bytes(&parts[0][2..].as_bytes(), 16)?;
            let mname = ModuleName::new(addr, self.env.symbol_pool().make(parts[1]));
            parts = &parts[2..];
            Some(self.env.find_module(&mname)?).and_then(skip_dependency)
        } else {
            None
        };
        let try_func_or_struct = |module: &ModuleEnv<'_>, name: Symbol, is_qualified: bool| {
            // Below we only resolve a simple name to a function if it is followed by a ( or <.
            // Otherwise we get too many false positives where names are resolved to functions
            // but were actual fields.
            if module.find_struct(name).is_some()
                || ((is_qualified || is_followed_by_open) && module.find_function(name).is_some())
            {
                Some(self.label_for_module_item(&module, name))
            } else {
                None
            }
        };
        let parts_sym = parts
            .iter()
            .map(|p| self.env.symbol_pool().make(p))
            .collect_vec();

        match (module_opt, parts_sym.len()) {
            (Some(module), 0) => Some(self.label_for_module(&module)),
            (Some(module), 1) => try_func_or_struct(&module, parts_sym[0], true),
            (None, 0) => None,
            (None, 1) => {
                // A simple name. Resolve either to module or to item in current module.
                if let Some(module) = self
                    .env
                    .find_module_by_name(parts_sym[0])
                    .and_then(skip_dependency)
                {
                    Some(self.label_for_module(&module))
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
                    self.env
                        .find_module_by_name(parts_sym[0])
                        .and_then(skip_dependency)
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

    /// Return the link label for a module.
    fn label_for_module(&self, module_env: &ModuleEnv<'_>) -> String {
        module_env
            .get_name()
            .display_full(self.env.symbol_pool())
            .to_string()
            .replace("::", "_")
    }

    /// Return the link label for an item in a module.
    fn label_for_module_item(&self, module_env: &ModuleEnv<'_>, item: Symbol) -> String {
        format!(
            "{}_{}",
            self.label_for_module(module_env),
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

    /// Fixes indentation of source code as obtained via original source.
    /// Typically code has the first line unindented because location tracking starts
    /// at the first keyword of the item (e.g. `public fun`), but subsequent lines are then
    /// indented. This uses a heuristic by taking the indentation of the last line as a basis.
    fn fix_indent(&self, s: &str) -> String {
        let lines = s.lines().collect_vec();
        if lines.is_empty() {
            return s.to_owned();
        }
        let last_line = lines[lines.len() - 1];
        let last_indent = last_line.len() - last_line.trim_start().len();
        lines
            .iter()
            .map(|l| {
                let mut i = 0;
                while i < last_indent && l.starts_with(' ') {
                    i += 1;
                }
                &l[i..]
            })
            .join("\n")
    }

    /// Repeats a string n times.
    fn repeat_str(&self, s: &str, n: usize) -> String {
        (0..n).map(|_| s).collect::<String>()
    }
}

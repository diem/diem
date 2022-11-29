// Copyright (c) The Diem Core Contributors
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use log::{debug, info, warn};

use codespan::{ByteIndex, Span};
use itertools::Itertools;
use move_model::{
    ast::{ModuleName, SpecBlockInfo, SpecBlockTarget},
    code_writer::{CodeWriter, CodeWriterLabel},
    emit, emitln,
    model::{
        AbilitySet, FunId, FunctionEnv, GlobalEnv, Loc, ModuleEnv, ModuleId, NamedConstantEnv,
        Parameter, QualifiedId, StructEnv, TypeParameter,
    },
    symbol::Symbol,
    ty::TypeDisplayContext,
};
use num::BigUint;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    rc::Rc,
};

const KEYWORDS: &[&str] = &[
    "abort",
    "acquires",
    "as",
    "break",
    "const",
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
    "friend",
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
    "aborts_if",
    "aborts_with",
    "apply",
    "assert",
    "assume",
    "decreases",
    "except",
    "forall",
    "global",
    "modifies",
    "ensures",
    "exists",
    "include",
    "mut",
    "old",
    "pragma",
    "pack",
    "unpack",
    "update",
    "requires",
    "schema",
    "to",
    "with",
    "where",
];

/// The maximum number of subheadings that are allowed
const MAX_SUBSECTIONS: usize = 6;

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
    /// A list of paths to files containing templates for root documents for the generated
    /// documentation.
    ///
    /// A root document is a markdown file which contains placeholders for generated
    /// documentation content. It is also processed following the same rules than
    /// documentation comments in Move, including creation of cross-references and
    /// Move code highlighting.
    ///
    /// A placeholder is a single line starting with a markdown quotation marker
    /// of the following form:
    ///
    /// ```notrust
    /// > {{move-include NAME_OF_MODULE_OR_SCRIPT}}
    /// > {{move-toc}}
    /// > {{move-index}}
    /// ```
    ///
    /// These lines will be replaced by the generated content of the module or script,
    /// or a table of contents, respectively.
    ///
    /// For a module or script which is included in the root document, no
    /// separate file is generated. References between the included and the standalone
    /// module/script content work transparently.
    pub root_doc_templates: Vec<String>,
    /// An optional file containing reference definitions. The content of this file will
    /// be added to each generated markdown doc.
    pub references_file: Option<String>,
    /// Whether to include dependency diagrams in the generated docs.
    pub include_dep_diagrams: bool,
    /// Whether to include call diagrams in the generated docs.
    pub include_call_diagrams: bool,
}

impl Default for DocgenOptions {
    fn default() -> Self {
        Self {
            section_level_start: 1,
            include_private_fun: true,
            include_specs: true,
            specs_inlined: true,
            include_impl: true,
            toc_depth: 3,
            collapsed_sections: true,
            output_directory: "doc".to_string(),
            doc_path: vec!["doc".to_string()],
            root_doc_templates: vec![],
            references_file: None,
            include_dep_diagrams: false,
            include_call_diagrams: false,
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
    /// A list of file names and output generated for those files.
    output: Vec<(String, String)>,
    /// Map from module id to information about this module.
    infos: BTreeMap<ModuleId, ModuleInfo>,
    /// Current code writer.
    writer: CodeWriter,
    /// Current module.
    current_module: Option<ModuleEnv<'env>>,
    /// A counter for labels.
    label_counter: RefCell<usize>,
    /// A mapping from location to spec item defined at this location.
    loc_to_spec_item_map: BTreeMap<Loc, Symbol>,
    /// A table-of-contents list.
    toc: RefCell<Vec<(usize, TocEntry)>>,
    /// The current section next
    section_nest: RefCell<usize>,
    /// The last user provided (via an explicit # header) section nest.
    last_root_section_nest: RefCell<usize>,
}

/// Information about the generated documentation for a specific script or module.
#[derive(Debug, Default, Clone)]
struct ModuleInfo {
    /// The file in which the generated content for this module is located. This has a path
    /// relative to the `options.output_directory`.
    target_file: String,
    /// The label in this file.
    label: String,
    /// Whether this module is included in another document instead of living in its own file.
    /// Among others, we do not generate table-of-contents for included modules.
    is_included: bool,
}

/// A table-of-contents entry.
#[derive(Debug, Default, Clone)]
struct TocEntry {
    label: String,
    title: String,
}

/// An element of the parsed root document template.
enum TemplateElement {
    Text(String),
    IncludeModule(String),
    IncludeToc,
    Index,
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
            infos: Default::default(),
            writer: CodeWriter::new(env.unknown_loc()),
            label_counter: RefCell::new(0),
            current_module: None,
            loc_to_spec_item_map: Default::default(),
            toc: RefCell::new(Default::default()),
            section_nest: RefCell::new(0),
            last_root_section_nest: RefCell::new(0),
        }
    }

    /// Generate document contents, returning pairs of output file names and generated contents.
    pub fn gen(mut self) -> Vec<(String, String)> {
        // Compute missing information about schemas.
        self.compute_declared_schemas();

        // If there is a root templates, parse them.
        let root_templates = self
            .options
            .root_doc_templates
            .iter()
            .filter_map(|file_name| {
                let root_out_name = PathBuf::from(file_name)
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace("_template", "");
                match self.parse_root_template(file_name) {
                    Ok(elements) => Some((root_out_name, elements)),
                    Err(_) => {
                        self.env.error(
                            &self.env.unknown_loc(),
                            &format!("cannot read root template `{}`", file_name),
                        );
                        None
                    }
                }
            })
            .collect_vec();

        // Compute module infos.
        self.compute_module_infos(&root_templates);

        // Expand all root templates.
        for (out_file, elements) in root_templates {
            self.expand_root_template(&out_file, elements);
        }

        // Generate documentation for standalone modules which are not included in the templates.
        for (id, info) in self.infos.clone() {
            let m = self.env.get_module(id);
            if !info.is_included && m.is_target() {
                self.gen_module(&m, &info);
                let mut path = PathBuf::from(&self.options.output_directory);
                path.push(info.target_file);
                self.output.push((
                    path.to_string_lossy().to_string(),
                    self.writer.extract_result(),
                ));
            }
        }

        // If there is a references_file, append it's content to each generated output.
        if let Some(fname) = &self.options.references_file {
            let mut content = String::new();
            if File::open(fname)
                .and_then(|mut file| file.read_to_string(&mut content))
                .is_ok()
            {
                let trimmed_content = content.trim();
                if !trimmed_content.is_empty() {
                    for (_, out) in self.output.iter_mut() {
                        out.push_str("\n\n");
                        out.push_str(trimmed_content);
                        out.push('\n');
                    }
                }
            } else {
                self.env.error(
                    &self.env.unknown_loc(),
                    &format!("cannot read references file `{}`", fname),
                );
            }
        }

        self.output
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

    /// Parse a root template.
    fn parse_root_template(&self, file_name: &str) -> anyhow::Result<Vec<TemplateElement>> {
        static REX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(
                r"(?xm)^\s*>\s*\{\{
                ( (?P<include>move-include\s+(?P<include_name>\w+))
                | (?P<toc>move-toc)
                | (?P<index>move-index)
                )\s*}}.*$",
            )
            .unwrap()
        });
        let mut content = String::new();
        let mut file = File::open(file_name)?;
        file.read_to_string(&mut content)?;
        let mut at = 0;
        let mut res = vec![];
        while let Some(cap) = REX.captures(&content[at..]) {
            let start = cap.get(0).unwrap().start();
            let end = cap.get(0).unwrap().end();
            if start > 0 {
                res.push(TemplateElement::Text(content[at..at + start].to_string()));
            }
            if cap.name("include").is_some() {
                let name = cap.name("include_name").unwrap().as_str();
                res.push(TemplateElement::IncludeModule(name.to_string()));
            } else if cap.name("toc").is_some() {
                res.push(TemplateElement::IncludeToc);
            } else if cap.name("index").is_some() {
                res.push(TemplateElement::Index);
            } else {
                unreachable!("regex misbehavior");
            }
            at += end;
        }
        if at < content.len() {
            res.push(TemplateElement::Text(content[at..].to_string()));
        }
        Ok(res)
    }

    /// Expand the root template.
    fn expand_root_template(&mut self, output_file_name: &str, elements: Vec<TemplateElement>) {
        self.writer = CodeWriter::new(self.env.unknown_loc());
        *self.label_counter.borrow_mut() = 0;
        let mut toc_label = None;
        for elem in elements {
            match elem {
                TemplateElement::Text(str) => self.doc_text_for_root(&str),
                TemplateElement::IncludeModule(name) => {
                    if let Some(module_env) = self
                        .env
                        .find_module_by_name(self.env.symbol_pool().make(&name))
                    {
                        let info = self
                            .infos
                            .get(&module_env.get_id())
                            .expect("module defined")
                            .clone();
                        assert!(info.is_included);
                        // Generate the module content in place, adjusting the section nest to
                        // the last user provided one. This will nest the module underneath
                        // whatever section is in the template.
                        let saved_nest = *self.section_nest.borrow();
                        *self.section_nest.borrow_mut() = *self.last_root_section_nest.borrow() + 1;
                        self.gen_module(&module_env, &info);
                        *self.section_nest.borrow_mut() = saved_nest;
                    } else {
                        emitln!(self.writer, "> undefined move-include `{}`", name);
                    }
                }
                TemplateElement::IncludeToc => {
                    if toc_label.is_none() {
                        toc_label = Some(self.writer.create_label());
                    } else {
                        // CodeWriter can only maintain one label at a time.
                        emitln!(self.writer, ">> duplicate move-toc (technical restriction)");
                    }
                }
                TemplateElement::Index => {
                    self.gen_index();
                }
            }
        }
        if let Some(label) = toc_label {
            // Insert the TOC.
            self.gen_toc(label);
        }

        // Add result to output.
        self.output.push((
            self.make_file_in_out_dir(output_file_name),
            self.writer.extract_result(),
        ));
    }

    /// Compute ModuleInfo for all modules, considering root template content.
    fn compute_module_infos(&mut self, templates: &[(String, Vec<TemplateElement>)]) {
        let mut out_dir = self.options.output_directory.to_string();
        if out_dir.is_empty() {
            out_dir = ".".to_string();
        }
        let log = |m: &ModuleEnv<'_>, i: &ModuleInfo| {
            info!(
                "{} `{}` in file `{}/{}` {}",
                Self::module_modifier(m.get_name()),
                m.get_name().display_full(m.symbol_pool()),
                out_dir,
                i.target_file,
                if !m.is_target() {
                    "exists"
                } else {
                    "will be generated"
                }
            );
        };
        // First process infos for modules included via template.
        let mut included = BTreeSet::new();
        for (template_out_file, elements) in templates {
            for element in elements {
                if let TemplateElement::IncludeModule(name) = element {
                    // TODO: currently we only support simple names, we may want to add support for
                    //   address qualification.
                    let sym = self.env.symbol_pool().make(name.as_str());
                    if let Some(module_env) = self.env.find_module_by_name(sym) {
                        let info = ModuleInfo {
                            target_file: template_out_file.to_string(),
                            label: self.make_label_for_module(&module_env),
                            is_included: true,
                        };
                        log(&module_env, &info);
                        self.infos.insert(module_env.get_id(), info);
                        included.insert(module_env.get_id());
                    } else {
                        // If this is not defined, we continue anyway and will not expand
                        // the placeholder in the generated root doc (following common template
                        // practice).
                    }
                }
            }
        }
        // Now process infos for all remaining modules.
        for m in self.env.get_modules() {
            if !included.contains(&m.get_id()) {
                if let Some(file_name) = self.compute_output_file(&m) {
                    let info = ModuleInfo {
                        target_file: file_name,
                        label: self.make_label_for_module(&m),
                        is_included: false,
                    };
                    log(&m, &info);
                    self.infos.insert(m.get_id(), info);
                }
            }
        }
    }

    fn module_modifier(name: &ModuleName) -> &str {
        if name.is_script() {
            "Script"
        } else {
            "Module"
        }
    }

    /// Computes file location for a module. This considers if the module is a dependency
    /// and if so attempts to locate already generated documentation for it.
    fn compute_output_file(&self, module_env: &ModuleEnv<'env>) -> Option<String> {
        let output_path = PathBuf::from(&self.options.output_directory);
        let file_name = PathBuf::from(module_env.get_source_path())
            .with_extension("md")
            .file_name()
            .expect("file name")
            .to_os_string();
        if !module_env.is_target() {
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
        }
    }

    /// Make a file name in the output directory.
    fn make_file_in_out_dir(&self, name: &str) -> String {
        let mut path = PathBuf::from(&self.options.output_directory);
        path.push(name);
        path.to_string_lossy().to_string()
    }

    /// Make path relative to other path.
    fn path_relative_to(&self, path: &Path, to: &Path) -> PathBuf {
        if path.is_absolute() || to.is_absolute() {
            path.to_path_buf()
        } else {
            let mut result = PathBuf::new();
            for _ in to.components() {
                result.push("..");
            }
            result.join(path)
        }
    }

    /// Generates documentation for a module. The result is written into the current code
    /// writer. Writer and other state is initialized if this module is standalone.
    fn gen_module(&mut self, module_env: &ModuleEnv<'env>, info: &ModuleInfo) {
        if !info.is_included {
            // (Re-) initialize state for this module.
            self.writer = CodeWriter::new(self.env.unknown_loc());
            self.toc = RefCell::new(Default::default());
            *self.section_nest.borrow_mut() = 0;
            *self.label_counter.borrow_mut() = 0;
        }
        self.current_module = Some(module_env.clone());

        // Initialize location to spec item map.
        self.loc_to_spec_item_map.clear();
        for (_, sfun) in module_env.get_spec_funs() {
            self.loc_to_spec_item_map
                .insert(sfun.loc.clone(), sfun.name);
        }
        for (_, svar) in module_env.get_spec_vars() {
            self.loc_to_spec_item_map
                .insert(svar.loc.clone(), svar.name);
        }

        // Print header
        self.section_header(
            &format!(
                "{} `{}`",
                Self::module_modifier(module_env.get_name()),
                module_env.get_name().display_full(module_env.symbol_pool())
            ),
            &info.label,
        );

        self.increment_section_nest();

        // Document module overview.
        self.doc_text(module_env.get_doc());

        // If this is a standalone doc, generate TOC header.
        let toc_label = if !info.is_included {
            Some(self.gen_toc_header())
        } else {
            None
        };

        // Generate usage information.
        // We currently only include modules used in bytecode -- including specs
        // creates a large usage list because of schema inclusion quickly pulling in
        // many modules.
        self.begin_code();
        let used_modules = module_env
            .get_used_modules(/*include_specs*/ false)
            .iter()
            .filter(|id| **id != module_env.get_id())
            .map(|id| {
                module_env
                    .env
                    .get_module(*id)
                    .get_name()
                    .display_full(module_env.symbol_pool())
                    .to_string()
            })
            .sorted();
        for used_module in used_modules {
            self.code_text(&format!("use {};", used_module));
        }
        self.end_code();

        if self.options.include_dep_diagrams {
            let module_name = module_env.get_name().display(module_env.symbol_pool());
            self.gen_dependency_diagram(module_env.get_id(), true);
            self.begin_collapsed(&format!(
                "Show all the modules that \"{}\" depends on directly or indirectly",
                module_name
            ));
            self.image(&format!("img/{}_forward_dep.svg", module_name));
            self.end_collapsed();

            if !module_env.is_script_module() {
                self.gen_dependency_diagram(module_env.get_id(), false);
                self.begin_collapsed(&format!(
                    "Show all the modules that depend on \"{}\" directly or indirectly",
                    module_name
                ));
                self.image(&format!("img/{}_backward_dep.svg", module_name));
                self.end_collapsed();
            }
        }

        let spec_block_map = self.organize_spec_blocks(module_env);

        if !module_env.get_structs().count() > 0 {
            for s in module_env
                .get_structs()
                .sorted_by(|a, b| Ord::cmp(&a.get_loc(), &b.get_loc()))
            {
                self.gen_struct(&spec_block_map, &s);
            }
        }

        if module_env.get_named_constant_count() > 0 {
            // Introduce a Constant section
            self.gen_named_constants();
        }

        let funs = module_env
            .get_functions()
            .filter(|f| self.options.include_private_fun || f.is_exposed())
            .sorted_by(|a, b| Ord::cmp(&a.get_loc(), &b.get_loc()))
            .collect_vec();
        if !funs.is_empty() {
            for f in funs {
                self.gen_function(&spec_block_map, &f);
            }
        }

        if !self.options.specs_inlined {
            self.gen_spec_section(module_env, &spec_block_map);
        } else {
            match spec_block_map.get(&SpecBlockTarget::Module) {
                Some(blocks) if !blocks.is_empty() => {
                    self.section_header(
                        "Module Specification",
                        &self.label_for_section("Module Specification"),
                    );
                    self.increment_section_nest();
                    self.gen_spec_blocks(module_env, "", &SpecBlockTarget::Module, &spec_block_map);
                    self.decrement_section_nest();
                }
                _ => {}
            }
        }

        self.decrement_section_nest();

        // Generate table of contents if this is standalone.
        if let Some(label) = toc_label {
            self.gen_toc(label);
        }
    }

    /// Generate a static call diagram (.svg) starting from the given function.
    fn gen_call_diagram(&self, fun_id: QualifiedId<FunId>) {
        let fun_env = self.env.get_function(fun_id);
        let name_of = |env: &FunctionEnv| {
            if fun_env.module_env.get_id() == env.module_env.get_id() {
                env.get_simple_name_string()
            } else {
                Rc::from(format!("\"{}\"", env.get_name_string()))
            }
        };

        let mut dot_src_lines: Vec<String> = vec!["digraph G {".to_string()];
        let mut visited: BTreeSet<QualifiedId<FunId>> = BTreeSet::new();
        let mut queue: VecDeque<QualifiedId<FunId>> = VecDeque::new();

        visited.insert(fun_id);
        queue.push_back(fun_id);

        while let Some(id) = queue.pop_front() {
            let caller_env = self.env.get_function(id);
            let caller_name = name_of(&caller_env);
            let callee_list = caller_env.get_called_functions();

            if fun_env.module_env.get_id() == caller_env.module_env.get_id() {
                dot_src_lines.push(format!("\t{}", caller_name));
            } else {
                let module_name = caller_env
                    .module_env
                    .get_name()
                    .display(caller_env.module_env.symbol_pool());
                dot_src_lines.push(format!("\tsubgraph cluster_{} {{", module_name));
                dot_src_lines.push(format!("\t\tlabel = \"{}\";", module_name));
                dot_src_lines.push(format!(
                    "\t\t{}[label=\"{}\"]",
                    caller_name,
                    caller_env.get_simple_name_string()
                ));
                dot_src_lines.push("\t}".to_string());
            }

            for callee_id in callee_list.iter() {
                let callee_env = self.env.get_function(*callee_id);
                let callee_name = name_of(&callee_env);
                dot_src_lines.push(format!("\t{} -> {}", caller_name, callee_name));
                if !visited.contains(callee_id) {
                    visited.insert(*callee_id);
                    queue.push_back(*callee_id);
                }
            }
        }
        dot_src_lines.push("}".to_string());

        let out_file_path = PathBuf::from(&self.options.output_directory)
            .join("img")
            .join(format!(
                "{}_call_graph.svg",
                fun_env.get_name_string().to_string().replace("::", "_")
            ));

        self.gen_svg_file(&out_file_path, &dot_src_lines.join("\n"));
    }

    /// Generate a forward (or backward) dependency diagram (.svg) for the given module.
    fn gen_dependency_diagram(&self, module_id: ModuleId, is_forward: bool) {
        let module_env = self.env.get_module(module_id);
        let module_name = module_env.get_name().display(module_env.symbol_pool());

        let mut dot_src_lines: Vec<String> = vec!["digraph G {".to_string()];
        let mut visited: BTreeSet<ModuleId> = BTreeSet::new();
        let mut queue: VecDeque<ModuleId> = VecDeque::new();

        visited.insert(module_id);
        queue.push_back(module_id);

        while let Some(id) = queue.pop_front() {
            let mod_env = self.env.get_module(id);
            let mod_name = mod_env.get_name().display(mod_env.symbol_pool());
            let dep_list = if is_forward {
                mod_env.get_used_modules(false)
            } else {
                mod_env.get_using_modules(false)
            };
            dot_src_lines.push(format!("\t{}", mod_name));
            for dep_id in dep_list.iter().filter(|dep_id| **dep_id != id) {
                let dep_env = self.env.get_module(*dep_id);
                let dep_name = dep_env.get_name().display(dep_env.symbol_pool());
                if is_forward {
                    dot_src_lines.push(format!("\t{} -> {}", mod_name, dep_name));
                } else {
                    dot_src_lines.push(format!("\t{} -> {}", dep_name, mod_name));
                }
                if !visited.contains(dep_id) {
                    visited.insert(*dep_id);
                    queue.push_back(*dep_id);
                }
            }
        }
        dot_src_lines.push("}".to_string());

        let out_file_path = PathBuf::from(&self.options.output_directory)
            .join("img")
            .join(format!(
                "{}_{}_dep.svg",
                module_name,
                (if is_forward { "forward" } else { "backward" })
            ));

        self.gen_svg_file(&out_file_path, &dot_src_lines.join("\n"));
    }

    /// Execute the external tool "dot" with doc_src as input to generate a .svg image file.
    fn gen_svg_file(&self, out_file_path: &Path, dot_src: &str) {
        if let Err(e) = fs::create_dir_all(out_file_path.parent().unwrap()) {
            self.env.error(
                &self.env.unknown_loc(),
                &format!("cannot create a directory for images ({})", e),
            );
            return;
        }

        let mut child = match Command::new("dot")
            .arg("-Tsvg")
            .args(&["-o", out_file_path.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                self.env.error(
                    &self.env.unknown_loc(),
                    &format!("The Graphviz tool \"dot\" is not available. {}", e),
                );
                return;
            }
        };

        if let Err(e) = child
            .stdin
            .as_mut()
            .ok_or("Child process stdin has not been captured!")
            .unwrap()
            .write_all(dot_src.as_bytes())
        {
            self.env.error(&self.env.unknown_loc(), &format!("{}", e));
            return;
        }

        match child.wait_with_output() {
            Ok(output) => {
                if !output.status.success() {
                    self.env.error(
                        &self.env.unknown_loc(),
                        &format!(
                            "dot failed to generate {}\n{}",
                            out_file_path.to_str().unwrap(),
                            dot_src
                        ),
                    );
                }
            }
            Err(e) => {
                self.env.error(&self.env.unknown_loc(), &format!("{}", e));
            }
        }
    }

    /// Generate header for TOC, returning label where we can later insert the content after
    /// file generation is done.
    fn gen_toc_header(&mut self) -> CodeWriterLabel {
        // Create label where we later can insert the TOC
        emitln!(self.writer);
        let toc_label = self.writer.create_label();
        emitln!(self.writer);
        toc_label
    }

    /// Generate table of content and insert it at label.
    fn gen_toc(&mut self, label: CodeWriterLabel) {
        // We put this into a separate code writer and insert its content at the label.
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
                self.item_text(&format!("[{}](#{})", entry.title, entry.label));
            }
            while level > 0 {
                self.end_items();
                self.writer.unindent();
                level -= 1;
            }
            // Insert the result at label.
            self.writer
                .process_result(|s| writer.insert_at_label(label, s));
        }
        self.writer = writer;
    }

    /// Generate an index of all modules and scripts in the context. This includes generated
    /// ones and those which are only dependencies.
    fn gen_index(&self) {
        // Sort all modules and script by simple name. (Perhaps we should include addresses?)
        let sorted_infos = self.infos.iter().sorted_by(|(id1, _), (id2, _)| {
            let name = |id: ModuleId| {
                self.env
                    .symbol_pool()
                    .string(self.env.get_module(id).get_name().name())
            };
            Ord::cmp(name(**id1).as_str(), name(**id2).as_str())
        });
        self.begin_items();
        for (id, _) in sorted_infos {
            let module_env = self.env.get_module(*id);
            self.item_text(&format!(
                "[`{}`]({})",
                module_env.get_name().display_full(module_env.symbol_pool()),
                self.ref_for_module(&module_env)
            ))
        }
        self.end_items();
    }

    /// Generates documentation for all named constants.
    fn gen_named_constants(&self) {
        self.section_header("Constants", &self.label_for_section("Constants"));
        self.increment_section_nest();
        for const_env in self.current_module.as_ref().unwrap().get_named_constants() {
            self.label(&self.label_for_module_item(&const_env.module_env, const_env.get_name()));
            self.doc_text(const_env.get_doc());
            self.code_block(&self.named_constant_display(&const_env));
        }

        self.decrement_section_nest();
    }

    /// Generates documentation for a struct.
    fn gen_struct(&self, spec_block_map: &SpecBlockMap<'_>, struct_env: &StructEnv<'_>) {
        let name = struct_env.get_name();
        self.section_header(
            &self.struct_title(struct_env),
            &self.label_for_module_item(&struct_env.module_env, name),
        );
        self.increment_section_nest();
        self.doc_text(struct_env.get_doc());
        self.code_block(&self.struct_header_display(struct_env));

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

    /// Returns "Struct `N`" or "Resource `N`".
    fn struct_title(&self, struct_env: &StructEnv<'_>) -> String {
        // NOTE(mengxu): although we no longer declare structs with the `resource` keyword, it
        // might be helpful in keeping `Resource N` in struct title as the boogie translator still
        // depends on the `is_resource()` predicate to add additional functions to structs declared
        // with the `key` ability.
        format!(
            "{} `{}`",
            if struct_env.has_memory() {
                "Resource"
            } else {
                "Struct"
            },
            self.name_string(struct_env.get_name())
        )
    }

    /// Generates declaration for named constant
    fn named_constant_display(&self, const_env: &NamedConstantEnv<'_>) -> String {
        let name = self.name_string(const_env.get_name());
        format!(
            "const {}: {} = {};",
            name,
            const_env.get_type().display(&TypeDisplayContext::WithEnv {
                env: self.env,
                type_param_names: None,
            }),
            const_env.get_value(),
        )
    }

    /// Generates code signature for a struct.
    fn struct_header_display(&self, struct_env: &StructEnv<'_>) -> String {
        let name = self.name_string(struct_env.get_name());
        let type_params = self.type_parameter_list_display(&struct_env.get_named_type_parameters());
        let ability_tokens = self.ability_tokens(struct_env.get_abilities());
        if ability_tokens.is_empty() {
            format!("struct {}{}", name, type_params)
        } else {
            format!(
                "struct {}{} has {}",
                name,
                type_params,
                ability_tokens.join(", ")
            )
        }
    }

    fn gen_struct_fields(&self, struct_env: &StructEnv<'_>) {
        let tctx = self.type_display_context_for_struct(struct_env);
        self.begin_definitions();
        for field in struct_env.get_fields() {
            self.definition_text(
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
        let is_script = func_env.module_env.is_script_module();
        let name = func_env.get_name();
        if !is_script {
            self.section_header(
                &format!("Function `{}`", self.name_string(name)),
                &self.label_for_module_item(&func_env.module_env, name),
            );
            self.increment_section_nest();
        }
        self.doc_text(func_env.get_doc());
        let sig = self.function_header_display(func_env);
        self.code_block(&sig);
        if self.options.include_impl {
            self.begin_collapsed("Implementation");
            self.code_block(&self.get_source_with_indent(&func_env.get_loc()));
            self.end_collapsed();
        }
        if self.options.specs_inlined {
            self.gen_spec_blocks(
                &func_env.module_env,
                "Specification",
                &SpecBlockTarget::Function(func_env.module_env.get_id(), func_env.get_id()),
                spec_block_map,
            )
        }
        if self.options.include_call_diagrams {
            let func_name = func_env.get_simple_name_string();
            self.gen_call_diagram(func_env.get_qualified_id());
            self.begin_collapsed(&format!(
                "Show all the functions that \"{}\" calls",
                &func_name
            ));
            self.image(&format!(
                "img/{}_call_graph.svg",
                func_env.get_name_string().to_string().replace("::", "_")
            ));
            self.end_collapsed();
        }
        if !is_script {
            self.decrement_section_nest();
        }
    }

    /// Generates documentation for a function signature.
    fn function_header_display(&self, func_env: &FunctionEnv<'_>) -> String {
        let name = self.name_string(func_env.get_name());
        let tctx = &self.type_display_context_for_fun(func_env);
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
            func_env.visibility_str(),
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
            self.doc_text(self.env.get_doc(&block.loc));
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
                        self.code_text(&schema_header);
                        self.writer.indent();
                    }
                    *in_code = true;
                }
            };
            let end_code = |in_code: &mut bool| {
                if *in_code {
                    if is_schema {
                        self.writer.unindent();
                        self.code_text("}");
                    }
                    self.end_code();
                    *in_code = false;
                }
            };
            for loc in &block.member_locs {
                let doc = self.env.get_doc(loc);
                if !doc.is_empty() {
                    end_code(&mut in_code);
                    self.doc_text(doc);
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
                self.code_text(&self.get_source_with_indent(loc));
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
        let mut last_block_end: Option<ByteIndex> = None;
        for block in module_env.get_spec_block_infos() {
            let may_merge_with_current = match &block.target {
                SpecBlockTarget::Schema(..) => true,
                SpecBlockTarget::Module
                    if !block.member_locs.is_empty() || !self.is_single_liner(&block.loc) =>
                {
                    // This is a bit of a hack: if spec module is on a single line,
                    // we consider it as a marker to switch doc context back to module level,
                    // otherwise (the case in this branch), we merge it with the predecessor.
                    true
                }
                _ => false,
            };
            if !may_merge_with_current
                || last_block_end.is_none()
                || self.has_move_code_inbetween(last_block_end.unwrap(), block.loc.span().start())
            {
                // Switch target if it's not a schema or module, or if there is any move code between
                // this block and the last one.
                current_target = block.target.clone();
            }
            last_block_end = Some(block.loc.span().end());
            result
                .entry(current_target.clone())
                .or_insert_with(Vec::new)
                .push(block);
        }
        result
    }

    /// Returns true if there is any move code (function or struct declaration)
    /// between the start and end positions.
    fn has_move_code_inbetween(&self, start: ByteIndex, end: ByteIndex) -> bool {
        // TODO(wrwg): this might be a bit of inefficient for larger modules, and
        //   we may want to precompute some of this if it becomses a bottleneck.
        if let Some(m) = &self.current_module {
            m.get_functions()
                .map(|f| f.get_loc())
                .chain(m.get_structs().map(|s| s.get_loc()))
                .any(|loc| {
                    let p = loc.span().start();
                    p >= start && p < end
                })
        } else {
            false
        }
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
        let section_label = self.label_for_section("Specification");
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
                    &self.struct_title(&struct_env),
                    &format!("{}_{}", section_label, name),
                );
                self.code_block(&self.struct_header_display(&struct_env));
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
                self.code_block(&self.function_header_display(&func_env));
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

    /// Collect tokens in an ability set
    fn ability_tokens(&self, abilities: AbilitySet) -> Vec<&'static str> {
        let mut ability_tokens = vec![];
        if abilities.has_copy() {
            ability_tokens.push("copy");
        }
        if abilities.has_drop() {
            ability_tokens.push("drop");
        }
        if abilities.has_store() {
            ability_tokens.push("store");
        }
        if abilities.has_key() {
            ability_tokens.push("key");
        }
        ability_tokens
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
        if usize::saturating_add(self.options.section_level_start, level) > MAX_SUBSECTIONS {
            panic!("Maximum number of subheadings exceeded with heading: {}", s)
        }
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

    /// Includes the image in the given path.
    fn image(&self, path: &str) {
        emitln!(self.writer);
        emitln!(self.writer, "![]({})", path);
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
        emitln!(self.writer);
        if self.options.collapsed_sections {
            emitln!(self.writer, "<details>");
            emitln!(self.writer, "<summary>{}</summary>", summary);
        } else {
            emitln!(self.writer, "##### {}", summary);
        }
        emitln!(self.writer);
    }

    /// Ends a collapsed section.
    fn end_collapsed(&self) {
        if self.options.collapsed_sections {
            emitln!(self.writer);
            emitln!(self.writer, "</details>");
        }
    }

    /// Outputs documentation text.
    fn doc_text_general(&self, for_root: bool, text: &str) {
        for line in self.decorate_text(text).lines() {
            let line = line.trim();
            if line.starts_with('#') {
                let mut i = 1;
                while line[i..].starts_with('#') {
                    i += 1;
                    self.increment_section_nest();
                }
                let header = line[i..].trim_start();
                if for_root {
                    *self.last_root_section_nest.borrow_mut() = *self.section_nest.borrow();
                }
                self.section_header(header, &self.label_for_section(header));
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

    fn doc_text_for_root(&self, text: &str) {
        self.doc_text_general(true, text)
    }

    fn doc_text(&self, text: &str) {
        self.doc_text_general(false, text)
    }

    /// Makes a label from a string.
    fn make_label_from_str(&self, s: &str) -> String {
        format!("@{}", s.replace(' ', "_"))
    }

    /// Decorates documentation text, identifying code fragments and decorating them
    /// as code. Code blocks in comments are untouched.
    fn decorate_text(&self, text: &str) -> String {
        let mut decorated_text = String::new();
        let mut chars = text.chars();
        let non_code_filter = |chr: &char| *chr != '`';

        while let Some(chr) = chars.next() {
            if chr == '`' {
                // See if this is the start of a code block.
                let is_start_of_code_block = chars.take_while_ref(|chr| *chr == '`').count() > 0;
                if is_start_of_code_block {
                    // Code block -- don't create a <code>text</code> for this.
                    decorated_text += "```";
                } else {
                    // inside inline code section. Eagerly consume/match this '`'
                    let code = chars.take_while_ref(non_code_filter).collect::<String>();
                    // consume the remaining '`'. Report an error if we find an unmatched '`'.
                    assert!(
                                            chars.next() == Some('`'),
                                            "Missing backtick found in {} while generating documentation for the following text: \"{}\"",
                                            self.current_module.as_ref().unwrap().get_name().display_full(self.env.symbol_pool()), text,
                                        );
                    use std::fmt::Write as _;
                    let _ = write!(decorated_text, "<code>{}</code>", self.decorate_code(&code));
                }
            } else {
                decorated_text.push(chr);
                decorated_text.extend(chars.take_while_ref(non_code_filter))
            }
        }
        decorated_text
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
    fn code_text(&self, code: &str) {
        emitln!(self.writer, &self.decorate_code(code));
    }

    /// Decorates a code fragment, for use in an html block. Replaces < and >, bolds keywords and
    /// tries to resolve and cross-link references.
    fn decorate_code(&self, code: &str) -> String {
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
                    } else if let Some(label) = self.resolve_to_label(s, is_call) {
                        format!("<a href=\"{}\">{}</a>", label, s)
                    } else {
                        "".to_owned()
                    }
                } else {
                    "".to_owned()
                }
            };
            if replacement.is_empty() {
                r += &code[at..at + cap.get(0).unwrap().end()].replace('<', "&lt;");
            } else {
                r += &code[at..at + cap.get(0).unwrap().start()];
                r += &replacement;
                if let Some(m) = cap.name("call") {
                    // Append the call or generic open we may have also matched to distinguish
                    // a simple name from a function call or generic instantiation. Need to
                    // replace the `<` as well.
                    r += &m.as_str().replace('<', "&lt;");
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
    fn resolve_to_label(&self, mut s: &str, is_followed_by_open: bool) -> Option<String> {
        // For clarity in documentation, we allow `script::` or `module::` as a prefix.
        // However, right now it will be ignored for resolution.
        let lower_s = s.to_lowercase();
        if lower_s.starts_with("script::") {
            s = &s["script::".len()..]
        } else if lower_s.starts_with("module::") {
            s = &s["module::".len()..]
        }
        let parts_data: Vec<&str> = s.splitn(3, "::").collect();
        let mut parts = parts_data.as_slice();
        let module_opt = if parts[0].starts_with("0x") {
            if parts.len() == 1 {
                // Cannot resolve.
                return None;
            }
            let addr = BigUint::parse_bytes(parts[0][2..].as_bytes(), 16)?;
            let mname = ModuleName::new(addr, self.env.symbol_pool().make(parts[1]));
            parts = &parts[2..];
            Some(self.env.find_module(&mname)?)
        } else {
            None
        };
        let try_func_struct_or_const =
            |module: &ModuleEnv<'_>, name: Symbol, is_qualified: bool| {
                // Below we only resolve a simple name to a hyperref if it is followed by a ( or <,
                // or if it is a named constant in the module.
                // Otherwise we get too many false positives where names are resolved to functions
                // but are actually fields.
                if module.find_struct(name).is_some()
                    || module.find_named_constant(name).is_some()
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
                    Some(self.ref_for_module_item(module, name))
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
            (Some(module), 1) => try_func_struct_or_const(&module, parts_sym[0], true),
            (None, 0) => None,
            (None, 1) => {
                // A simple name. Resolve either to module or to item in current module.
                if let Some(module) = self.env.find_module_by_name(parts_sym[0]) {
                    Some(self.ref_for_module(&module))
                } else if let Some(module) = &self.current_module {
                    try_func_struct_or_const(module, parts_sym[0], false)
                } else {
                    None
                }
            }
            (None, 2) => {
                // A qualified name, but without the address. This must be an item in a module
                // denoted by the first name.
                let module_opt = if parts[0] == "Self" {
                    self.current_module.as_ref().cloned()
                } else {
                    self.env.find_module_by_name(parts_sym[0])
                };
                if let Some(module) = module_opt {
                    try_func_struct_or_const(&module, parts_sym[1], true)
                } else {
                    None
                }
            }
            (_, _) => None,
        }
    }

    /// Create label for a module.
    fn make_label_for_module(&self, module_env: &ModuleEnv<'_>) -> String {
        module_env
            .get_name()
            .display_full(self.env.symbol_pool())
            .to_string()
            .replace("::", "_")
    }

    /// Return the label for a module.
    fn label_for_module(&self, module_env: &ModuleEnv<'_>) -> &str {
        if let Some(info) = self.infos.get(&module_env.get_id()) {
            &info.label
        } else {
            ""
        }
    }

    /// Return the reference for a module.
    fn ref_for_module(&self, module_env: &ModuleEnv<'_>) -> String {
        if let Some(info) = self.infos.get(&module_env.get_id()) {
            format!("{}#{}", info.target_file, info.label)
        } else {
            "".to_string()
        }
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

    /// Create a unique label for a section header.
    fn label_for_section(&self, title: &str) -> String {
        let counter = *self.label_counter.borrow();
        *self.label_counter.borrow_mut() += 1;
        self.make_label_from_str(&format!("{} {}", title, counter))
    }

    /// Shortcut for code_block in a module context.
    fn code_block(&self, code: &str) {
        self.begin_code();
        self.code_text(code);
        self.end_code();
    }

    /// Begin an itemized list.
    fn begin_items(&self) {}

    /// End an itemized list.
    fn end_items(&self) {}

    /// Emit an item.
    fn item_text(&self, text: &str) {
        emitln!(self.writer, "-  {}", text);
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
    fn definition_text(&self, term: &str, def: &str) {
        emitln!(self.writer, "<dt>\n{}\n</dt>", self.decorate_text(term));
        emitln!(self.writer, "<dd>\n{}\n</dd>", self.decorate_text(def));
    }

    /// Display a type parameter.
    fn type_parameter_display(&self, tp: &TypeParameter) -> String {
        let ability_tokens = self.ability_tokens(tp.1 .0);
        if ability_tokens.is_empty() {
            self.name_string(tp.0).to_string()
        } else {
            format!("{}: {}", self.name_string(tp.0), ability_tokens.join(", "))
        }
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
            let mut indent = source_before.len() - newl_at - 1;
            if indent >= 4 && source_before.ends_with("spec ") {
                // Special case for `spec define` and similar constructs.
                indent -= 4;
            }
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

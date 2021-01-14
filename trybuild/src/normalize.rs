#[cfg(test)]
#[path = "tests.rs"]
mod tests;

use std::path::Path;

#[derive(Copy, Clone)]
pub struct Context<'a> {
    pub krate: &'a str,
    pub source_dir: &'a Path,
    pub workspace: &'a Path,
}

pub fn trim<S: AsRef<[u8]>>(output: S) -> String {
    let bytes = output.as_ref();
    let mut normalized = String::from_utf8_lossy(bytes).to_string();

    let len = normalized.trim_end().len();
    normalized.truncate(len);

    if !normalized.is_empty() {
        normalized.push('\n');
    }

    normalized
}

/// For a given compiler output, produces the set of saved outputs against which
/// the compiler's output would be considered correct. If the test's saved
/// stderr file is identical to any one of these variations, the test will pass.
///
/// This is a set rather than just one normalized output in order to avoid
/// breaking existing tests when introducing new normalization steps. Someone
/// may have saved stderr snapshots with an older version of trybuild, and those
/// tests need to continue to pass with newer versions of trybuild.
///
/// There is one "preferred" variation which is what we print when the stderr
/// file is absent or not a match.
pub fn diagnostics(output: Vec<u8>, context: Context) -> Variations {
    let mut from_bytes = String::from_utf8_lossy(&output).to_string();
    from_bytes = from_bytes.replace("\r\n", "\n");

    let variations = [
        Basic,
        StripCouldNotCompile,
        StripCouldNotCompile2,
        StripForMoreInformation,
        StripForMoreInformation2,
        DirBackslash,
        TrimEnd,
        RustLib,
        TypeDirBackslash,
        WorkspaceLines,
    ]
    .iter()
    .map(|normalization| apply(&from_bytes, *normalization, context))
    .collect();

    Variations { variations }
}

pub struct Variations {
    variations: Vec<String>,
}

impl Variations {
    pub fn preferred(&self) -> &str {
        self.variations.last().unwrap()
    }

    pub fn any<F: FnMut(&str) -> bool>(&self, mut f: F) -> bool {
        self.variations.iter().any(|stderr| f(stderr))
    }
}

#[derive(PartialOrd, PartialEq, Copy, Clone)]
enum Normalization {
    Basic,
    StripCouldNotCompile,
    StripCouldNotCompile2,
    StripForMoreInformation,
    StripForMoreInformation2,
    DirBackslash,
    TrimEnd,
    RustLib,
    TypeDirBackslash,
    WorkspaceLines,
    // New normalization steps are to be inserted here at the end so that any
    // snapshots saved before your normalization change remain passing.
}

use self::Normalization::*;

fn apply(original: &str, normalization: Normalization, context: Context) -> String {
    let mut normalized = String::new();

    let lines: Vec<&str> = original.lines().collect();
    let mut filter = Filter {
        all_lines: &lines,
        normalization,
        context,
        hide_numbers: 0,
    };
    for i in 0..lines.len() {
        if let Some(line) = filter.apply(i) {
            normalized += &line;
            if !normalized.ends_with("\n\n") {
                normalized.push('\n');
            }
        }
    }

    trim(normalized)
}

struct Filter<'a> {
    all_lines: &'a [&'a str],
    normalization: Normalization,
    context: Context<'a>,
    hide_numbers: usize,
}

impl<'a> Filter<'a> {
    fn apply(&mut self, index: usize) -> Option<String> {
        let mut line = self.all_lines[index].to_owned();

        if self.hide_numbers > 0 {
            hide_leading_numbers(&mut line);
            self.hide_numbers -= 1;
        }

        if line.trim_start().starts_with("--> ") {
            if let Some(cut_end) = line.rfind(&['/', '\\'][..]) {
                let cut_start = line.find('>').unwrap() + 2;
                return Some(line[..cut_start].to_owned() + "$DIR/" + &line[cut_end + 1..]);
            }
        }

        if line.trim_start().starts_with("::: ") {
            let mut other_crate = false;
            let line_lower = line.to_ascii_lowercase();
            let workspace_pat = self
                .context
                .workspace
                .to_string_lossy()
                .to_ascii_lowercase();
            if let Some(i) = line_lower.find(&workspace_pat) {
                line.replace_range(i..i + workspace_pat.len(), "$WORKSPACE");
                other_crate = true;
            }
            let mut line = line.replace('\\', "/");
            if self.normalization >= RustLib {
                if let Some(pos) = line.find("/rustlib/src/rust/src/") {
                    // ::: $RUST/src/libstd/net/ip.rs:83:1
                    line.replace_range(line.find("::: ").unwrap() + 4..pos + 17, "$RUST");
                    other_crate = true;
                } else if let Some(pos) = line.find("/rustlib/src/rust/library/") {
                    // ::: $RUST/std/src/net/ip.rs:83:1
                    line.replace_range(line.find("::: ").unwrap() + 4..pos + 25, "$RUST");
                    other_crate = true;
                }
            }
            if other_crate && self.normalization >= WorkspaceLines {
                // Blank out line numbers for this particular error since rustc
                // tends to reach into code from outside of the test case. The
                // test stderr shouldn't need to be updated every time we touch
                // those files.
                hide_trailing_numbers(&mut line);
                self.hide_numbers = 2;
                for (fwd, next_line) in self.all_lines[index + 1..].iter().take(6).enumerate() {
                    if next_line.trim_start().is_empty()
                        || next_line.contains(" required by this bound in `")
                    {
                        self.hide_numbers = fwd;
                        break;
                    }
                }
            }
            return Some(line);
        }

        if line.starts_with("error: aborting due to ") {
            return None;
        }

        if line == "To learn more, run the command again with --verbose." {
            return None;
        }

        if self.normalization >= StripCouldNotCompile {
            if line.starts_with("error: Could not compile `") {
                return None;
            }
        }

        if self.normalization >= StripCouldNotCompile2 {
            if line.starts_with("error: could not compile `") {
                return None;
            }
        }

        if self.normalization >= StripForMoreInformation {
            if line.starts_with("For more information about this error, try `rustc --explain") {
                return None;
            }
        }

        if self.normalization >= StripForMoreInformation2 {
            if line.starts_with("Some errors have detailed explanations:") {
                return None;
            }
            if line.starts_with("For more information about an error, try `rustc --explain") {
                return None;
            }
        }

        if self.normalization >= DirBackslash {
            // https://github.com/dtolnay/trybuild/issues/66
            let source_dir_with_backslash =
                self.context.source_dir.to_string_lossy().into_owned() + "\\";
            line = replace_case_insensitive(&line, &source_dir_with_backslash, "$DIR/");
        }

        if self.normalization >= TrimEnd {
            line.truncate(line.trim_end().len());
        }

        if self.normalization >= TypeDirBackslash {
            if line
                .trim_start()
                .starts_with("= note: required because it appears within the type")
            {
                line = line.replace('\\', "/");
            }
        }

        line = line.replace(self.context.krate, "$CRATE");
        line = replace_case_insensitive(&line, &self.context.source_dir.to_string_lossy(), "$DIR");
        line = replace_case_insensitive(
            &line,
            &self.context.workspace.to_string_lossy(),
            "$WORKSPACE",
        );

        Some(line)
    }
}

// "10 | T: Send,"  ->  "   | T: Send,"
fn hide_leading_numbers(line: &mut String) {
    let n = line.bytes().take_while(u8::is_ascii_digit).count();
    for i in 0..n {
        line.replace_range(i..i + 1, " ");
    }
}

// "main.rs:22:29"  ->  "main.rs"
fn hide_trailing_numbers(line: &mut String) {
    for _ in 0..2 {
        let digits = line.bytes().rev().take_while(u8::is_ascii_digit).count();
        if digits == 0 || !line[..line.len() - digits].ends_with(':') {
            return;
        }
        line.truncate(line.len() - digits - 1);
    }
}

fn replace_case_insensitive(line: &str, pattern: &str, replacement: &str) -> String {
    let line_lower = line.to_ascii_lowercase();
    let pattern_lower = pattern.to_ascii_lowercase();
    let mut replaced = String::with_capacity(line.len());
    for (i, keep) in line_lower.split(&pattern_lower).enumerate() {
        if i > 0 {
            replaced.push_str(replacement);
        }
        let begin = replaced.len() - i * replacement.len() + i * pattern.len();
        let end = begin + keep.len();
        replaced.push_str(&line[begin..end]);
    }
    replaced
}

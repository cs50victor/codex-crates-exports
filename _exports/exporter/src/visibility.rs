use anyhow::{Context, Result, ensure};
use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Item, Visibility};

struct Edit {
    start: usize,
    end: usize,
    text: &'static str,
}

fn offset(source: &str, span: Span, end: bool) -> Result<usize> {
    let position = if end { span.end() } else { span.start() };
    let line_start = source
        .split_inclusive('\n')
        .take(position.line.saturating_sub(1))
        .map(str::len)
        .sum::<usize>();
    let line = source.get(line_start..).context("invalid span line")?;
    let column = line
        .char_indices()
        .nth(position.column)
        .map_or(line.len(), |(index, _)| index);
    Ok(line_start + column)
}

fn collect(source: &str, items: &[Item], edits: &mut Vec<Edit>) -> Result<()> {
    for item in items {
        if let Item::Mod(item) = item {
            if item.attrs.iter().any(|attribute| {
                attribute.path().is_ident("cfg")
                    && matches!(&attribute.meta, syn::Meta::List(list) if list.tokens.to_string() == "test")
            }) {
                continue;
            }
            match &item.vis {
                Visibility::Public(_) => {}
                Visibility::Inherited => {
                    let start = offset(source, item.mod_token.span(), false)?;
                    edits.push(Edit {
                        start,
                        end: start,
                        text: "pub ",
                    });
                }
                Visibility::Restricted(_) => edits.push(Edit {
                    start: offset(source, item.vis.span(), false)?,
                    end: offset(source, item.vis.span(), true)?,
                    text: "pub",
                }),
            }
            if let Some((_, items)) = &item.content {
                collect(source, items, edits)?;
            }
        }
    }
    Ok(())
}

pub fn rewrite(source: &str) -> Result<String> {
    let syntax = syn::parse_file(source).context("parse upstream Rust")?;
    let mut edits = Vec::new();
    collect(source, &syntax.items, &mut edits)?;
    edits.sort_by_key(|edit| (edit.start, edit.end));
    for pair in edits.windows(2) {
        ensure!(pair[0].end <= pair[1].start, "overlapping visibility edits");
    }
    let mut output = source.to_owned();
    for edit in edits.into_iter().rev() {
        ensure!(
            output.is_char_boundary(edit.start) && output.is_char_boundary(edit.end),
            "invalid UTF-8 span"
        );
        output.replace_range(edit.start..edit.end, edit.text);
    }
    syn::parse_file(&output).context("parse exported Rust")?;
    Ok(output)
}

pub fn rewrite_binary_entrypoint(source: &str) -> Result<String> {
    let source = rewrite(source)?;
    let syntax = syn::parse_file(&source)?;
    let mut edits = Vec::new();
    let mut found = false;
    for item in &syntax.items {
        if let Item::Fn(item) = item
            && item.sig.ident == "main"
        {
            found = true;
            if matches!(item.vis, Visibility::Inherited) {
                edits.push(offset(&source, item.sig.span(), false)?);
            }
        }
    }
    ensure!(
        found,
        "binary adapter has no source-defined main entrypoint"
    );
    let mut output = source;
    for start in edits.into_iter().rev() {
        output.insert_str(start, "pub ");
    }
    syn::parse_file(&output).context("parse binary adapter")?;
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_module_paths_and_preserves_encapsulation() {
        let source = "mod a { pub(crate) mod b { pub struct S(u8); impl S { fn new() -> Self { Self(1) } } } }";
        assert_eq!(
            rewrite(source).unwrap(),
            "pub mod a { pub mod b { pub struct S(u8); impl S { fn new() -> Self { Self(1) } } } }"
        );
    }

    #[test]
    fn preserves_trait_implementations_macros_and_local_items() {
        let source = "macro_rules! hidden { () => { mod hidden {} } } fn run() { mod local {} } struct S; trait T { fn f(); } impl T for S { fn f() {} }";
        assert_eq!(rewrite(source).unwrap(), source);
    }

    #[test]
    fn preserves_attributes_comments_and_unicode() {
        let source = "// é λ\n#[cfg(unix)]\npub(super) mod café { mod inner {} }\n";
        assert_eq!(
            rewrite(source).unwrap(),
            "// é λ\n#[cfg(unix)]\npub mod café { pub mod inner {} }\n"
        );
    }

    #[test]
    fn handles_all_visibility_forms_idempotently() {
        let source = "pub mod a { pub(self) mod b {} pub(in crate::a) mod c {} }";
        let once = rewrite(source).unwrap();
        assert_eq!(once, "pub mod a { pub mod b {} pub mod c {} }");
        assert_eq!(rewrite(&once).unwrap(), once);
    }

    #[test]
    fn rejects_invalid_source() {
        assert!(rewrite("fn (").is_err());
    }

    #[test]
    fn exposes_only_binary_entrypoints_and_preserves_cfg_tests() {
        let source = "#[cfg(unix)] fn main() {} #[cfg(windows)] fn main() {} fn helper() {} #[cfg(test)] mod tests {}";
        let output = rewrite_binary_entrypoint(source).unwrap();
        assert_eq!(
            output,
            "#[cfg(unix)] pub fn main() {} #[cfg(windows)] pub fn main() {} fn helper() {} #[cfg(test)] mod tests {}"
        );
        assert_eq!(rewrite_binary_entrypoint(&output).unwrap(), output);
        assert!(rewrite_binary_entrypoint("fn helper() {}").is_err());
    }
}

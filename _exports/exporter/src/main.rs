mod visibility;

use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use toml_edit::{DocumentMut, InlineTable, Item, Value, value};

fn metadata(workspace: &Path) -> Result<Json> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(workspace.join("Cargo.toml"))
        .output()?;
    ensure!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn dependency_table(item: &Item) -> Result<InlineTable> {
    match item {
        Item::Value(Value::String(version)) => {
            let mut table = InlineTable::new();
            table.insert("version", Value::from(version.value().as_str()));
            Ok(table)
        }
        Item::Value(Value::InlineTable(table)) => Ok(table.clone()),
        Item::Table(table) => Ok(table.clone().into_inline_table()),
        _ => bail!("unsupported dependency syntax"),
    }
}

fn rewrite_dependencies(item: &mut Item, patches: &[(String, InlineTable)]) -> Result<()> {
    let Some(table) = item.as_table_like_mut() else {
        return Ok(());
    };
    for (key, item) in table.iter_mut() {
        if ["dependencies", "dev-dependencies", "build-dependencies"].contains(&key.get()) {
            let dependencies = item.as_table_like_mut().context("dependency table")?;
            for (name, item) in dependencies.iter_mut() {
                let mut dependency = dependency_table(item)?;
                if dependency.get("workspace").and_then(Value::as_bool) == Some(true) {
                    continue;
                }
                let package = dependency
                    .get("package")
                    .and_then(Value::as_str)
                    .unwrap_or(name.get());
                if let Some((_, patch)) = patches.iter().find(|(name, _)| name == package) {
                    for key in [
                        "version",
                        "registry",
                        "registry-index",
                        "path",
                        "git",
                        "rev",
                        "branch",
                        "tag",
                    ] {
                        dependency.remove(key);
                    }
                    for (key, val) in patch {
                        dependency.insert(key, val.clone());
                    }
                    *item = Item::Value(Value::InlineTable(dependency));
                }
            }
        } else if key != "patch" {
            rewrite_dependencies(item, patches)?;
        }
    }
    Ok(())
}

fn rust_files(root: &Path, files: &mut BTreeSet<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let kind = entry.file_type()?;
        if kind.is_symlink() {
            continue;
        }
        if kind.is_dir() {
            let name = entry.file_name();
            if ![
                "tests",
                "fixtures",
                "test_fixtures",
                "testdata",
                "snapshots",
                "target",
            ]
            .iter()
            .any(|skip| name == *skip)
            {
                rust_files(&path, files)?;
            }
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.insert(path);
        }
    }
    Ok(())
}

fn export(root: &Path) -> Result<()> {
    let ignore_path = root.join(".gitignore");
    let mut ignore = fs::read_to_string(&ignore_path).unwrap_or_default();
    for pattern in ["/target/", "/_exports/exporter/target/"] {
        if !ignore.lines().any(|line| line == pattern) {
            ignore.push_str(&format!("\n{pattern}\n"));
        }
    }
    fs::write(ignore_path, ignore)?;
    let workspace = root.join("codex-rs");
    let metadata = metadata(&workspace)?;
    let mut root_manifest =
        fs::read_to_string(workspace.join("Cargo.toml"))?.parse::<DocumentMut>()?;
    let mut patches = Vec::new();
    if let Some(table) = root_manifest["patch"]["crates-io"].as_table_like() {
        for (name, item) in table.iter() {
            let patch = dependency_table(item)?;
            ensure!(
                patch.contains_key("git") && patch.contains_key("rev"),
                "patch {name} must use a pinned Git revision"
            );
            patches.push((name.to_owned(), patch));
        }
    }
    rewrite_dependencies(root_manifest.as_item_mut(), &patches)?;
    fs::write(workspace.join("Cargo.toml"), root_manifest.to_string())?;
    let members = metadata["workspace_members"]
        .as_array()
        .context("workspace members")?;
    let packages = metadata["packages"].as_array().context("packages")?;
    let mut files = BTreeSet::new();
    let mut binary_entrypoints = BTreeSet::new();
    let mut catalog = Vec::new();
    for package in packages
        .iter()
        .filter(|package| members.contains(&package["id"]))
    {
        let manifest = Path::new(package["manifest_path"].as_str().context("manifest path")?);
        let mut document = fs::read_to_string(manifest)?.parse::<DocumentMut>()?;
        rewrite_dependencies(document.as_item_mut(), &patches)?;
        let targets = package["targets"].as_array().context("targets")?;
        let proc_macro = targets.iter().any(|target| {
            target["kind"]
                .as_array()
                .is_some_and(|kinds| kinds.contains(&json!("proc-macro")))
        });
        let library = targets.iter().find(|target| {
            target["kind"].as_array().is_some_and(|kinds| {
                kinds.iter().any(|kind| {
                    ["lib", "rlib", "cdylib", "staticlib", "proc-macro"]
                        .iter()
                        .any(|name| kind == name)
                })
            })
        });
        let generated_library = library.is_none()
            || document
                .get("package")
                .and_then(|item| item.get("metadata"))
                .and_then(|item| item.get("codex-crates-exports"))
                .and_then(|item| item.get("added-library-target"))
                .and_then(Item::as_bool)
                == Some(true);
        let target = match library {
            Some(target) => target,
            None => {
                let binaries: Vec<_> = targets
                    .iter()
                    .filter(|target| {
                        target["kind"]
                            .as_array()
                            .is_some_and(|kinds| kinds.contains(&json!("bin")))
                    })
                    .collect();
                ensure!(
                    binaries.len() == 1,
                    "binary-only package {} needs an explicit library adapter",
                    package["name"]
                );
                binaries[0]
            }
        };
        let source = Path::new(target["src_path"].as_str().context("target source")?);
        let directory = manifest.parent().context("package directory")?;
        if generated_library {
            binary_entrypoints.insert(source.to_path_buf());
            document["lib"]["path"] = value(
                source
                    .strip_prefix(directory)?
                    .to_str()
                    .context("UTF-8 source path")?,
            );
            document["lib"]["doctest"] = value(false);
            document["package"]["metadata"]["codex-crates-exports"]["added-library-target"] =
                value(true);
        }
        if !proc_macro {
            rust_files(source.parent().context("source directory")?, &mut files)?;
        }
        fs::write(manifest, document.to_string())?;
        catalog.push(json!({
            "name": package["name"],
            "path": directory.strip_prefix(root)?.to_str().context("UTF-8 package path")?,
            "kind": if proc_macro { "proc-macro" } else if generated_library { "binary-adapter" } else { "library" },
            "added_library_target": generated_library,
            "features": package["features"],
        }));
    }
    let mut rewritten = 0;
    for path in &files {
        let source = fs::read_to_string(path)?;
        let exported = if binary_entrypoints.contains(path) {
            visibility::rewrite_binary_entrypoint(&source)
        } else {
            visibility::rewrite(&source)
        }
        .with_context(|| format!("export {}", path.display()))?;
        if exported != source {
            fs::write(path, exported)?;
            rewritten += 1;
        }
    }
    catalog.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    fs::write(
        root.join("_exports/crates.json"),
        format!("{}\n", serde_json::to_string_pretty(&catalog)?),
    )?;
    println!(
        "Exported {} crates; parsed {} Rust files; rewrote {rewritten}",
        catalog.len(),
        files.len()
    );
    Ok(())
}

fn main() -> Result<()> {
    let root = std::env::args_os()
        .nth(1)
        .context("usage: codex-crates-exporter REPOSITORY")?;
    export(&fs::canonicalize(root)?)
}

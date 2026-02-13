use crate::{EMSCRIPTEN_VERSION, bail_on_err, root_dir};
use anyhow::Result;
use serde_json::Value;
use std::{fs, path::Path, process::Command};

enum FixtureRef<'a> {
    Tag(&'a str),
    Branch(&'a str),
}

impl<'a> FixtureRef<'a> {
    #[allow(clippy::use_self)]
    const fn new(tag: &'a str, branch: Option<&'a str>) -> FixtureRef<'a> {
        if let Some(b) = branch {
            Self::Branch(b)
        } else {
            Self::Tag(tag)
        }
    }

    const fn ref_type(&self) -> &'static str {
        match self {
            FixtureRef::Tag(_) => "tag",
            FixtureRef::Branch(_) => "branch",
        }
    }
}

impl std::fmt::Display for FixtureRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FixtureRef::Tag(tag) => write!(f, "{tag}"),
            FixtureRef::Branch(branch) => write!(f, "{branch}"),
        }
    }
}

fn current_ref_name(grammar_dir: &Path) -> Result<(String, Option<&'static str>)> {
    let tag_args = ["describe", "--tags", "--exact-match", "HEAD"];
    let branch_args = ["rev-parse", "--abbrev-ref", "HEAD"];

    for (args, ref_type) in [tag_args.as_ref(), branch_args.as_ref()]
        .iter()
        .zip(&["tag", "branch"])
    {
        let name_cmd = Command::new("git")
            .current_dir(grammar_dir)
            .args(*args)
            .output()?;
        let name = String::from_utf8_lossy(&name_cmd.stdout);
        let name = name.trim();
        if !name.is_empty() {
            return Ok((name.to_string(), Some(ref_type)));
        }
    }

    Ok(("<unknown>".to_string(), None))
}

pub fn run_fixtures() -> Result<()> {
    let fixtures_dir = root_dir().join("test").join("fixtures");
    let grammars_dir = fixtures_dir.join("grammars");
    let fixtures_path = fixtures_dir.join("fixtures.json");

    // grammar name, tag, [branch]
    let fixtures: Vec<(String, String, Option<String>)> =
        serde_json::from_str(&fs::read_to_string(&fixtures_path)?)?;

    for (grammar, tag, branch) in &fixtures {
        let grammar_dir = grammars_dir.join(grammar);
        let grammar_url = format!("https://github.com/tree-sitter/tree-sitter-{grammar}");
        let target_ref = FixtureRef::new(tag, branch.as_deref());

        println!("Fetching the {grammar} grammar...");

        if !grammar_dir.exists() {
            let mut command = Command::new("git");
            command.args([
                "clone",
                "--depth",
                "1",
                "--branch",
                &target_ref.to_string(),
                &grammar_url,
                &grammar_dir.to_string_lossy(),
            ]);
            bail_on_err(
                &command.spawn()?.wait_with_output()?,
                &format!("Failed to clone the {grammar} grammar"),
            )?;
        } else {
            let (current_ref, current_ref_type) = current_ref_name(&grammar_dir)?;
            if current_ref != target_ref.to_string() {
                println!(
                    "Updating {grammar} grammar from {} {current_ref} to {} {target_ref}...",
                    current_ref_type.unwrap_or("<unknown>"),
                    target_ref.ref_type(),
                );

                match target_ref {
                    FixtureRef::Branch(branch) => {
                        let mut fetch_cmd = Command::new("git");
                        fetch_cmd.current_dir(&grammar_dir).args([
                            "fetch",
                            "--update-shallow",
                            "origin",
                            &format!("+refs/heads/{branch}:refs/remotes/origin/{branch}"),
                        ]);
                        bail_on_err(
                            &fetch_cmd.spawn()?.wait_with_output()?,
                            &format!("Failed to fetch branch {branch}"),
                        )?;
                        let mut switch_cmd = Command::new("git");
                        switch_cmd
                            .current_dir(&grammar_dir)
                            .args(["switch", branch]);
                        bail_on_err(
                            &switch_cmd.spawn()?.wait_with_output()?,
                            &format!("Failed to checkout branch {branch}"),
                        )?;
                        let mut set_upstream_cmd = Command::new("git");
                        set_upstream_cmd.current_dir(&grammar_dir).args([
                            "branch",
                            "--set-upstream-to",
                            &format!("origin/{branch}"),
                            branch,
                        ]);
                        bail_on_err(
                            &set_upstream_cmd.spawn()?.wait_with_output()?,
                            &format!("Failed to set upstream for branch {branch}"),
                        )?;
                        let mut pull_cmd = Command::new("git");
                        pull_cmd
                            .current_dir(&grammar_dir)
                            .args(["pull", "origin", branch]);
                        bail_on_err(
                            &pull_cmd.spawn()?.wait_with_output()?,
                            &format!("Failed to pull latest from branch {branch}"),
                        )?;
                    }
                    FixtureRef::Tag(tag) => {
                        let mut fetch_command = Command::new("git");
                        fetch_command.current_dir(&grammar_dir).args([
                            "fetch",
                            "origin",
                            &format!("refs/tags/{tag}:refs/tags/{tag}"),
                        ]);
                        bail_on_err(
                            &fetch_command.spawn()?.wait_with_output()?,
                            &format!(
                                "Failed to fetch {} {target_ref} for {grammar} grammar",
                                target_ref.ref_type()
                            ),
                        )?;
                    }
                }

                let mut reset_command = Command::new("git");
                reset_command
                    .current_dir(&grammar_dir)
                    .args(["reset", "--hard", "HEAD"]);
                bail_on_err(
                    &reset_command.spawn()?.wait_with_output()?,
                    &format!("Failed to reset {grammar} grammar working tree"),
                )?;

                let mut checkout_command = Command::new("git");
                checkout_command
                    .current_dir(&grammar_dir)
                    .args(["checkout", &target_ref.to_string()]);
                bail_on_err(
                    &checkout_command.spawn()?.wait_with_output()?,
                    &format!(
                        "Failed to checkout {} {target_ref} for {grammar} grammar",
                        target_ref.ref_type()
                    ),
                )?;
            } else {
                println!(
                    "{grammar} grammar is already at {} {target_ref}",
                    target_ref.ref_type()
                );
            }
        }

        patch_fixture_package_json(&grammar_dir);
    }

    Ok(())
}

fn patch_package_json_value(pkg: &mut Value) {
    let Some(obj) = pkg.as_object_mut() else {
        return;
    };

    obj.insert("type".to_string(), Value::String("module".to_string()));
    obj.insert(
        "engines".to_string(),
        serde_json::json!({ "bun": ">=1.3.9" }),
    );

    if let Some(Value::Object(scripts)) = obj.get_mut("scripts") {
        for value in scripts.values_mut() {
            if let Value::String(s) = value
                && s.starts_with("node --test")
            {
                *s = s.replacen("node --test", "bun test", 1);
            }
        }
    }
}

fn patch_fixture_package_json(grammar_dir: &Path) {
    // Patch root package.json
    let package_json_path = grammar_dir.join("package.json");
    if package_json_path.exists()
        && let Ok(content) = fs::read_to_string(&package_json_path)
        && let Ok(mut pkg) = serde_json::from_str::<Value>(&content)
    {
        patch_package_json_value(&mut pkg);
        if let Ok(updated) = serde_json::to_string_pretty(&pkg) {
            let _ = fs::write(&package_json_path, updated + "\n");
        }
    }

    // Rewrite binding test files to ESM/bun:test
    patch_fixture_binding_tests(grammar_dir);

    // Delete package-lock.json
    let lock_path = grammar_dir.join("package-lock.json");
    if lock_path.exists() {
        let _ = fs::remove_file(&lock_path);
    }

    // Patch sub-package.json files (multi-grammar repos like php, typescript)
    if let Ok(entries) = fs::read_dir(grammar_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let sub_pkg_path = entry.path().join("package.json");
                if sub_pkg_path.exists()
                    && let Ok(content) = fs::read_to_string(&sub_pkg_path)
                    && let Ok(mut pkg) = serde_json::from_str::<Value>(&content)
                    && pkg.get("private").and_then(Value::as_bool) == Some(true)
                {
                    patch_package_json_value(&mut pkg);
                    if let Ok(updated) = serde_json::to_string_pretty(&pkg) {
                        let _ = fs::write(&sub_pkg_path, updated + "\n");
                    }
                }

                // Delete sub-directory package-lock.json too
                let sub_lock_path = entry.path().join("package-lock.json");
                if sub_lock_path.exists() {
                    let _ = fs::remove_file(&sub_lock_path);
                }
            }
        }
    }
}

/// Rewrite `bindings/node/binding_test.js` files from CJS/node:test to ESM/bun:test.
///
/// Handles three upstream patterns:
/// 1. Standard single-grammar tests (12 grammars) — single `test("can load grammar", ...)`
/// 2. PHP multi-grammar test — `describe`/`it` with named exports `{ php, php_only }`
/// 3. TypeScript multi-grammar test — two `test()` calls requiring `./typescript` and `./tsx`
fn patch_fixture_binding_tests(grammar_dir: &Path) {
    let test_path = grammar_dir.join("bindings/node/binding_test.js");
    let Ok(content) = fs::read_to_string(&test_path) else {
        return;
    };

    // Skip if already using bun:test
    if content.contains("bun:test") {
        return;
    }

    // Only patch files that use node:test
    if !content.contains("node:test") {
        return;
    }

    let patched = if content.contains("php_only") {
        // PHP multi-grammar pattern
        r#"import { describe, it, expect } from "bun:test";
import Parser from "tree-sitter";

const { php, php_only } = await import("../../index.js");

describe("PHP", () => {
  const parser = new Parser();
  parser.setLanguage(php);

  it("should be named php", () => {
    expect(parser.getLanguage().name).toBe("php");
  });

  it("should parse source code", () => {
    const sourceCode = "<?php echo 'Hello, World!';";
    const tree = parser.parse(sourceCode);
    expect(tree.rootNode.hasError).toBe(false);
  });
});

describe("PHP Only", () => {
  const parser = new Parser();
  parser.setLanguage(php_only);

  it("should be named php_only", () => {
    expect(parser.getLanguage().name).toBe("php_only");
  });

  it("should parse source code", () => {
    const sourceCode = "echo 'Hello, World!';";
    const tree = parser.parse(sourceCode);
    expect(tree.rootNode.hasError).toBe(false);
  });
});
"#
    } else if content.contains("./typescript") || content.contains("./tsx") {
        // TypeScript multi-grammar pattern
        r#"import { test, expect } from "bun:test";
import Parser from "tree-sitter";

test("can load TypeScript grammar", async () => {
  const parser = new Parser();
  const { default: language } = await import("./typescript/index.js");
  expect(() => parser.setLanguage(language)).not.toThrow();
});

test("can load TSX grammar", async () => {
  const parser = new Parser();
  const { default: language } = await import("./tsx/index.js");
  expect(() => parser.setLanguage(language)).not.toThrow();
});
"#
    } else {
        // Standard single-grammar pattern (most common)
        r#"import { test, expect } from "bun:test";
import Parser from "tree-sitter";

test("can load grammar", async () => {
  const parser = new Parser();
  const { default: language } = await import("./index.js");
  expect(() => parser.setLanguage(language)).not.toThrow();
});
"#
    };

    let _ = fs::write(&test_path, patched);
}

pub fn run_emscripten() -> Result<()> {
    let emscripten_dir = root_dir().join("target").join("emsdk");
    if emscripten_dir.exists() {
        println!("Emscripten SDK already exists");
        return Ok(());
    }
    println!("Cloning the Emscripten SDK...");

    let mut command = Command::new("git");
    command.args([
        "clone",
        "https://github.com/emscripten-core/emsdk.git",
        &emscripten_dir.to_string_lossy(),
    ]);
    bail_on_err(
        &command.spawn()?.wait_with_output()?,
        "Failed to clone the Emscripten SDK",
    )?;

    std::env::set_current_dir(&emscripten_dir)?;

    let emsdk = if cfg!(windows) {
        "emsdk.bat"
    } else {
        "./emsdk"
    };

    let mut command = Command::new(emsdk);
    command.args(["install", EMSCRIPTEN_VERSION]);
    bail_on_err(
        &command.spawn()?.wait_with_output()?,
        "Failed to install Emscripten",
    )?;

    let mut command = Command::new(emsdk);
    command.args(["activate", EMSCRIPTEN_VERSION]);
    bail_on_err(
        &command.spawn()?.wait_with_output()?,
        "Failed to activate Emscripten",
    )
}

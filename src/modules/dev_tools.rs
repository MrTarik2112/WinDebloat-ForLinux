use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;

pub struct DevToolsModule;

impl CleanModule for DevToolsModule {
    fn id(&self) -> &'static str { "dev_tools" }
    fn name(&self) -> &'static str { "Dev Tools" }
    fn category(&self) -> Category { Category::DevTools }
    fn description(&self) -> &'static str {
        "Clean development tool caches and build artifacts"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        // Node modules
        if let Some(home) = dirs::home_dir() {
            let node_paths = vec![
                home.join("node_modules"),
                home.join(".npm"),
            ];

            for node_path in node_paths {
                if node_path.exists() {
                    let size = crate::utils::disk::dir_size(&node_path);
                    if size > 10 * 1024 * 1024 {
                        items.push(CleanItem::new(
                            "node_modules",
                            &node_path.to_string_lossy(),
                            size,
                            false,
                            "Node.js dependencies",
                            &format!("rm -rf {}", node_path.display()),
                        ));
                    }
                }
            }

            // Cargo cache
            let cargo_home = home.join(".cargo/registry/cache");
            if cargo_home.exists() {
                let size = crate::utils::disk::dir_size(&cargo_home);
                if size > 50 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "cargo-cache",
                        &cargo_home.to_string_lossy(),
                        size,
                        true,
                        "Cargo registry cache",
                        "rm -rf ~/.cargo/registry/cache/*",
                    ));
                }
            }

            // Go cache
            let go_cache = home.join(".cache/go-build");
            if go_cache.exists() {
                let size = crate::utils::disk::dir_size(&go_cache);
                if size > 50 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "go-cache",
                        &go_cache.to_string_lossy(),
                        size,
                        true,
                        "Go build cache",
                        "go clean -cache",
                    ));
                }
            }

            // Maven cache
            let maven_cache = home.join(".m2/repository");
            if maven_cache.exists() {
                let size = crate::utils::disk::dir_size(&maven_cache);
                if size > 100 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        "maven-cache",
                        &maven_cache.to_string_lossy(),
                        size,
                        false,
                        "Maven local repository",
                        "rm -rf ~/.m2/repository/*",
                    ));
                }
            }

            // Python cache
            let python_dirs = vec![
                home.join(".cache/pip"),
                home.join(".cache/pip wheels"),
                home.join(".local/share/uv"),
            ];

            for py_dir in python_dirs {
                if py_dir.exists() {
                    let size = crate::utils::disk::dir_size(&py_dir);
                    if size > 10 * 1024 * 1024 {
                        items.push(CleanItem::new(
                            "pip-cache",
                            &py_dir.to_string_lossy(),
                            size,
                            true,
                            "Python pip cache",
                            &format!("rm -rf {}/*", py_dir.display()),
                        ));
                    }
                }
            }

            // Rust target directory
            let rust_target = home.join("rustup/toolchains");
            if rust_target.exists() {
                // Check multiple toolchains
                if let Ok(entries) = std::fs::read_dir(&rust_target) {
                    let toolchains: Vec<_> = entries.flatten().filter_map(|e| {
                        e.metadata().ok().and_then(|m| if m.is_dir() { Some(e.path()) } else { None })
                    }).collect();

                    if toolchains.len() > 1 {
                        let total_size: u64 = toolchains.iter()
                            .map(|p| crate::utils::disk::dir_size(p))
                            .sum();
                        
                        items.push(CleanItem::new(
                            "rust-toolchains",
                            &rust_target.to_string_lossy(),
                            total_size,
                            false,
                            &format!("{} Rust toolchains", toolchains.len()),
                            "rustup toolchain prune",
                        ));
                    }
                }
            }

            // Build artifacts in projects
            let build_patterns = vec!["target", "build", "dist", ".next", ".nuxt"];
            let project_dirs = vec![
                home.join("Projects"),
                home.join("workspace"),
                home.join("code"),
            ];

            for proj in project_dirs {
                if !proj.exists() {
                    continue;
                }
                
                for pattern in &build_patterns {
                    if let Ok(entries) = std::fs::read_dir(&proj) {
                        for entry in entries.flatten() {
                            let build_dir = entry.path().join(pattern);
                            if build_dir.exists() {
                                let size = crate::utils::disk::dir_size(&build_dir);
                                if size > 20 * 1024 * 1024 {
                                    items.push(CleanItem::new(
                                        "build-artifacts",
                                        &build_dir.to_string_lossy(),
                                        size,
                                        false,
                                        &format!("Build dir: {}", entry.file_name().to_string_lossy()),
                                        &format!("rm -rf {}", build_dir.display()),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        // System npm cache
        if let Ok(output) = std::process::Command::new("npm")
            .args(["cache", "ls", "--json"])
            .output()
        {
            let cache_size = String::from_utf8_lossy(&output.stdout).len() as u64 * 1000;
            if cache_size > 50 * 1024 * 1024 {
                items.push(CleanItem::new(
                    "npm-global-cache",
                    "/tmp/npm-cache",
                    cache_size,
                    true,
                    "NPM global cache",
                    "npm cache clean --force",
                ));
            }
        }

        Ok(items)
    }
}
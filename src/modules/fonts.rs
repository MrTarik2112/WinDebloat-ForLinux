use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use std::path::Path;
use walkdir::WalkDir;

pub struct FontsModule;

impl CleanModule for FontsModule {
    fn id(&self) -> &'static str { "fonts" }
    fn name(&self) -> &'static str { "Fonts" }
    fn category(&self) -> Category { Category::Fonts }
    fn description(&self) -> &'static str {
        "Find unused or duplicate fonts"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();
        
        let font_dirs = vec![
            dirs::home_dir().map(|h| h.join(".local/share/fonts")),
            dirs::home_dir().map(|h| h.join(".fonts")),
            Some(Path::new("/usr/local/share/fonts").to_path_buf()),
            Some(Path::new("/usr/share/fonts").to_path_buf()),
        ];

        let mut total_fonts = 0u64;
        let mut total_size = 0u64;
        let mut duplicates = Vec::new();
        let mut font_names: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for dir_opt in font_dirs.into_iter().flatten() {
            if !dir_opt.exists() {
                continue;
            }

            for entry in WalkDir::new(&dir_opt)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_lowercase();
                    if ext_str == "ttf" || ext_str == "otf" || ext_str == "ttc" {
                        total_fonts += 1;
                        if let Ok(meta) = path.metadata() {
                            total_size += meta.len();
                        }

                        let font_name = path.file_stem()
                            .map(|n| n.to_string_lossy().to_lowercase())
                            .unwrap_or_default();
                        
                        font_names.entry(font_name.clone())
                            .or_default()
                            .push(path.to_string_lossy().to_string());
                    }
                }
            }
        }

        for (_, paths) in font_names {
            if paths.len() > 1 {
                duplicates.push(paths);
            }
        }

        let duplicate_count: usize = duplicates.iter().map(|v| v.len() - 1).sum();
        let duplicate_size: u64 = duplicates.iter()
            .flat_map(|v| v.iter().skip(1))
            .filter_map(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .sum();

        if total_fonts > 0 {
            items.push(CleanItem::new(
                "user-fonts",
                &dirs::home_dir().map(|h| h.join(".local/share/fonts").to_string_lossy().to_string()).unwrap_or_default(),
                total_size,
                false,
                &format!("{} fonts installed", total_fonts),
                "Manual review recommended - use font-manager",
            ).with_file_count(total_fonts));
        }

        if duplicate_count > 0 {
            items.push(CleanItem::new(
                "duplicate-fonts",
                "~/.local/share/fonts",
                duplicate_size,
                false,
                &format!("{} duplicate fonts", duplicate_count),
                "Review duplicates manually",
            ).with_file_count(duplicate_count as u64));
        }

        Ok(items)
    }
}
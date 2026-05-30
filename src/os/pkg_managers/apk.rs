use crate::core::scanner::CleanItem;
use crate::utils::error::Result;

pub fn clean_cache() -> Result<Vec<CleanItem>> {
    let size = get_cache_size()?;
    if size > 0 {
        Ok(vec![CleanItem::new(
            "apk-cache",
            "/etc/apk/cache",
            size,
            true,
            "APK package cache",
            "apk cache clean",
        )])
    } else {
        Ok(vec![])
    }
}

pub fn clean_orphaned() -> Result<Vec<CleanItem>> {
    let mut items = Vec::new();
    let path = std::path::Path::new("/var/cache/apk");
    if path.exists() {
        let size = crate::utils::disk::dir_size(path);
        if size > 1024 * 1024 {
            items.push(CleanItem::new(
                "apk-orphaned",
                "/var/cache/apk",
                size,
                true,
                "APK cached packages",
                "rm -rf /var/cache/apk/*",
            ));
        }
    }
    Ok(items)
}

pub fn clean_old_kernels() -> Result<Vec<CleanItem>> {
    Ok(Vec::new())
}

fn get_cache_size() -> Result<u64> {
    let path = std::path::Path::new("/etc/apk/cache");
    if path.exists() {
        Ok(crate::utils::disk::dir_size(path))
    } else {
        Ok(0)
    }
}

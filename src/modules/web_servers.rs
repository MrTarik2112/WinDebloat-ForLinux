use super::CleanModule;
use crate::core::scanner::{Category, CleanItem};
use crate::os::distro::OSInfo;
use crate::utils::error::Result;
use crate::utils::disk::dir_size;
use std::path::Path;

pub struct WebServersModule;

impl CleanModule for WebServersModule {
    fn id(&self) -> &'static str { "web_servers" }
    fn name(&self) -> &'static str { "Web Servers" }
    fn category(&self) -> Category { Category::System }
    fn description(&self) -> &'static str {
        "Clean web server logs, access logs, and old backups"
    }

    fn scan(&self, _os: &OSInfo) -> Result<Vec<CleanItem>> {
        let mut items = Vec::new();

        let paths = vec![
            ("/var/log/apache2", "Apache access/error logs"),
            ("/var/log/httpd", "HTTPd logs"),
            ("/var/log/nginx", "Nginx access/error logs"),
            ("/var/log/lighttpd", "Lighttpd logs"),
            ("/var/log/caddy", "Caddy logs"),
            ("/var/log/tomcat", "Tomcat logs"),
            ("/var/log/jboss", "JBoss logs"),
        ];

        for (path, desc) in &paths {
            let p = Path::new(path);
            if p.exists() {
                let size = dir_size(p);
                if size > 10 * 1024 * 1024 {
                    items.push(CleanItem::new(
                        &format!("web-{}", path.replace('/', "-")),
                        path,
                        size,
                        true,
                        desc,
                        &format!("sudo find {} -name '*.log' -mtime +30 -delete 2>/dev/null", path),
                    ).with_category("web"));
                }
            }
        }

        Ok(items)
    }
}

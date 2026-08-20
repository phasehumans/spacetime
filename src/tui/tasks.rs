pub fn get_task_category_tag(task_id: &str) -> &'static str {
    if task_id.contains("nginx") || task_id.contains("port") || task_id.contains("dns") || task_id.contains("iptables") || task_id.contains("curl") || task_id.contains("jwt") {
        "[net]"
    } else if task_id.contains("git") {
        "[git]"
    } else if task_id.contains("perm") || task_id.contains("user") || task_id.contains("ssh") || task_id.contains("secret") || task_id.contains("cve") || task_id.contains("traversal") || task_id.contains("jail") || task_id.contains("chroot") {
        "[sec]"
    } else if task_id.contains("json") || task_id.contains("sqlite") || task_id.contains("base64") || task_id.contains("jq") || task_id.contains("awk") || task_id.contains("envsubst") || task_id.contains("redis") {
        "[data]"
    } else if task_id.contains("docker") || task_id.contains("process") || task_id.contains("calc") || task_id.contains("compilation") || task_id.contains("deadlock") || task_id.contains("systemd") || task_id.contains("shebang") || task_id.contains("venv") || task_id.contains("tmux") {
        "[dev]"
    } else if task_id.contains("file") || task_id.contains("tar") || task_id.contains("symlink") || task_id.contains("disk") || task_id.contains("rsync") {
        "[fs]"
    } else if task_id.contains("log") || task_id.contains("ip") || task_id.contains("traceback") {
        "[logs]"
    } else {
        "[os]"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_tagging() {
        assert_eq!(get_task_category_tag("001-nginx-config"), "[net]");
        assert_eq!(get_task_category_tag("004-port-conflict"), "[net]");
        assert_eq!(get_task_category_tag("005-resolve-git-conflict"), "[git]");
        assert_eq!(get_task_category_tag("012-fix-permissions"), "[sec]");
        assert_eq!(get_task_category_tag("022-rotate-leaked-secret"), "[sec]");
        assert_eq!(get_task_category_tag("023-fix-c-compilation-flags"), "[dev]");
        assert_eq!(get_task_category_tag("024-optimize-slow-sqlite"), "[data]");
        assert_eq!(get_task_category_tag("026-fix-dns-resolv-conf"), "[net]");
    }
}

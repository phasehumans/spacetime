pub fn get_task_category_tag(task_id: &str) -> &'static str {
    if task_id.contains("nginx") || task_id.contains("port") {
        "[net]"
    } else if task_id.contains("git") {
        "[git]"
    } else if task_id.contains("perm") || task_id.contains("user") || task_id.contains("ssh") {
        "[sec]"
    } else if task_id.contains("json") || task_id.contains("sqlite") || task_id.contains("base64") {
        "[data]"
    } else if task_id.contains("docker") || task_id.contains("process") {
        "[dev]"
    } else if task_id.contains("file") || task_id.contains("tar") || task_id.contains("symlink") {
        "[fs]"
    } else if task_id.contains("log") || task_id.contains("ip") {
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
        assert_eq!(get_task_category_tag("003-json-parsing"), "[data]");
        assert_eq!(get_task_category_tag("006-find-largest-file"), "[fs]");
        assert_eq!(get_task_category_tag("015-fix-dockerfile"), "[dev]");
        assert_eq!(get_task_category_tag("007-extract-log-errors"), "[logs]");
    }
}

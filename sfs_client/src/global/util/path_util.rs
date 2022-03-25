use crate::global::error_msg::error_msg;

static SEPERATOR: char = '/';

pub fn is_relative(path: &String) -> bool {
    path.len() != 0 && path.as_bytes()[0] as char != SEPERATOR
}
pub fn is_absolute(path: &String) -> bool {
    path.len() != 0 && path.as_bytes()[0] as char == SEPERATOR
}
pub fn has_trailing_slash(path: &String) -> bool {
    path.len() != 0 && path.ends_with(SEPERATOR)
}
pub fn has_trailing_endline(path: &String) -> bool {
    path.len() != 0 && path.ends_with("\n")
}
pub fn prepend_path(prefix: &String, raw_path: String) -> String {
    if !has_trailing_slash(&prefix) {
        format!("{}{}{}", prefix, SEPERATOR, raw_path)
    } else {
        format!("{}{}", prefix, raw_path)
    }
}
pub fn split_path(path: String) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut last_index = if is_absolute(&path) { 1 } else { 0 };
    for (i, &ch) in path.as_bytes().iter().enumerate() {
        if ch as char == SEPERATOR && i != 0 {
            tokens.push(path[last_index..i].to_string());
            last_index = i + 1;
        }
    }
    if last_index != path.len() {
        tokens.push(path[last_index..path.len()].to_string());
    }
    tokens
}
pub fn absolute_to_relative(mut root_path: String, absolute_path: String) -> String {
    if !is_absolute(&root_path) || !is_absolute(&absolute_path) {
        error_msg(
            "global::util::path_util::absolute_to_relative".to_string(),
            "argumants must be absolute path".to_string(),
        );
        return "".to_string();
    }
    if !absolute_path.starts_with(&root_path.clone()) {
        error_msg(
            "global::util::path_util::absolute_to_relative".to_string(),
            "the second path must start with the first path".to_string(),
        );
        return "".to_string();
    }
    if has_trailing_slash(&root_path) {
        root_path = root_path[0..root_path.len() - 1].to_string();
    }
    let sliced = absolute_path[root_path.len()..absolute_path.len()].to_string();
    if !is_absolute(&sliced) {
        error_msg(
            "global::util::path_util::absolute_to_relative".to_string(),
            "the first argument is not a complete path".to_string(),
        );
        return "".to_string();
    }
    sliced[1..sliced.len()].to_string()
}
pub fn dirname(path: &String) -> String {
    if path.len() > 1 && has_trailing_slash(&path) {
        error_msg(
            "global::util::path_util::dirname".to_string(),
            "path can't end with seperator".to_string(),
        );
        return "".to_string();
    }
    if path.len() == 1 && !is_absolute(&path) {
        error_msg(
            "global::util::path_util::dirname".to_string(),
            "path can't be a single character".to_string(),
        );
        return "".to_string();
    }
    if let Some(index) = path.rfind(SEPERATOR) {
        path[0..if index == 0 { 1 } else { index }].to_string()
    } else {
        error_msg(
            "global::util::path_util::dirname".to_string(),
            "there is no seperator in path".to_string(),
        );
        "".to_string()
    }
}

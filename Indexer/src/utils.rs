use std::collections::HashMap;


pub fn is_binary_extension(url: &str) -> bool {
    let binary_extensions = [
        ".exe", ".apk", ".dmg", ".pkg", ".deb", ".rpm",
        ".zip", ".rar", ".7z", ".tar", ".gz", ".bz2",
        ".pdf", ".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx",
        ".jpg", ".jpeg", ".png", ".gif", ".bmp", ".svg", ".ico",
        ".mp3", ".mp4", ".avi", ".mov", ".wmv", ".flv",
        ".bin", ".dll", ".so", ".dylib", ".class", ".jar"
    ];
    
    let url_lower = url.to_lowercase();
    binary_extensions.iter().any(|&ext| url_lower.ends_with(ext))
}
pub fn is_text_content(content_type: &str) -> bool {
    let allowed_types = [
        "text/html",
        "text/plain", 
        "text/xml",
        "application/xml",
        "application/xhtml+xml",
        "text/css",
        "text/javascript",
        "application/json",
        "application/ld+json"
    ];
    
    allowed_types.iter().any(|&allowed| content_type.starts_with(allowed))
}


pub struct Store<'a>{
    url:&'a str,
    pub tf_score:HashMap<String,i32>
}

impl<'a> Store<'a>{
    pub fn new(url:&'a str,tf_score:HashMap<String,i32>)->Store<'a>{
        Store { url, tf_score }
    }
}
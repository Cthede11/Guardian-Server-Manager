/// Input sanitization utilities
pub struct InputSanitizer;

impl InputSanitizer {
    pub fn sanitize_string(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || ".,!?@#$%^&*()_+-=[]{}|;':\",./<>?~`".contains(*c))
            .collect()
    }
    
    pub fn sanitize_filename(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '_' || *c == '-')
            .collect()
    }
    
    pub fn sanitize_path(input: &str) -> String {
        input
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '/' || *c == '.' || *c == '_' || *c == '-')
            .collect()
    }
    
    pub fn sanitize_html(input: &str) -> String {
        input
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&#x27;")
            .replace('&', "&amp;")
    }
    
    pub fn sanitize_js(input: &str) -> String {
        input
            .replace('\\', "\\\\")
            .replace('\'', "\\'")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    }
}

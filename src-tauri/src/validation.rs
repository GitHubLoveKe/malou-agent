//! 输入验证模块
//! 提供用户输入的安全验证和过滤功能



/// 验证错误类型
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ValidationError {}

/// 验证配置选项
#[derive(Debug, Clone)]
pub struct ValidationOptions {
    pub max_length: usize,
    pub min_length: usize,
    pub allow_empty: bool,
    pub allow_special_chars: bool,
    pub allow_html: bool,
    pub trim: bool,
}

impl Default for ValidationOptions {
    fn default() -> Self {
        Self {
            max_length: 10000,
            min_length: 1,
            allow_empty: false,
            allow_special_chars: true,
            allow_html: false,
            trim: true,
        }
    }
}

/// 特殊字符黑名单
const SPECIAL_CHARS_BLACKLIST: &[&str] = &[
    // 潜在的SQL注入字符
    ";", "--", "/*", "*/", "xp_", "sp_",
    // 潜在的XSS攻击字符
    "<script", "javascript:", "onload", "onerror", "onclick",
];

/// HTML标签黑名单
const HTML_TAGS_BLACKLIST: &[&str] = &[
    "script", "iframe", "object", "embed", "form", "input",
    "button", "select", "textarea", "meta", "link", "style",
];

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub message: String,
    pub sanitized_value: Option<String>,
}

/// 验证用户输入
pub fn validate_input(input: &str, options: ValidationOptions) -> ValidationResult {
    let mut sanitized_value = input.to_string();
    
    // 预处理输入
    if options.trim {
        sanitized_value = sanitized_value.trim().to_string();
    }
    
    // 检查空值
    if !options.allow_empty && sanitized_value.is_empty() {
        return ValidationResult {
            is_valid: false,
            message: "输入内容不能为空".to_string(),
            sanitized_value: None,
        };
    }
    
    // 检查最小长度
    if sanitized_value.len() < options.min_length {
        return ValidationResult {
            is_valid: false,
            message: format!("输入内容至少需要{}个字符", options.min_length),
            sanitized_value: None,
        };
    }
    
    // 检查最大长度
    if sanitized_value.len() > options.max_length {
        return ValidationResult {
            is_valid: false,
            message: format!("输入内容不能超过{}个字符", options.max_length),
            sanitized_value: None,
        };
    }
    
    // 检查特殊字符
    if !options.allow_special_chars {
        let has_special_chars = SPECIAL_CHARS_BLACKLIST.iter().any(|&char| 
            sanitized_value.to_lowercase().contains(char)
        );
        
        if has_special_chars {
            return ValidationResult {
                is_valid: false,
                message: "输入内容包含不允许的特殊字符".to_string(),
                sanitized_value: None,
            };
        }
    }
    
    // 检查HTML标签
    if !options.allow_html {
        let has_html_tags = HTML_TAGS_BLACKLIST.iter().any(|&tag| {
            let pattern = format!(r"<{}(\s|>)", tag);
            let re = regex::Regex::new(&pattern).unwrap();
            re.is_match(&sanitized_value)
        });
        
        if has_html_tags {
            // 移除HTML标签
            sanitized_value = sanitized_value.replace(r"<[^>]*>", "");
            
            // 如果移除了标签，检查是否还有有效内容
            if sanitized_value.trim().is_empty() {
                return ValidationResult {
                    is_valid: false,
                    message: "输入内容包含不安全的HTML标签".to_string(),
                    sanitized_value: None,
                };
            }
        }
    }
    
    // 额外的安全检查：URL和协议
    let url_pattern = regex::Regex::new(r"(https?://|ftp://|file://)").unwrap();
    if url_pattern.is_match(&sanitized_value) {
        // 检查是否为潜在的危险URL
        let dangerous_protocols = ["file:", "javascript:", "vbscript:"];
        let has_dangerous_url = dangerous_protocols.iter().any(|&protocol| 
            sanitized_value.to_lowercase().contains(protocol)
        );
        
        if has_dangerous_url {
            return ValidationResult {
                is_valid: false,
                message: "输入内容包含不安全的URL协议".to_string(),
                sanitized_value: None,
            };
        }
    }
    
    ValidationResult {
        is_valid: true,
        message: "输入验证通过".to_string(),
        sanitized_value: Some(sanitized_value),
    }
}

/// 验证聊天消息输入
pub fn validate_chat_message(content: &str) -> Result<String, ValidationError> {
    let options = ValidationOptions {
        max_length: 5000,
        min_length: 1,
        allow_empty: false,
        allow_special_chars: true,
        allow_html: false,
        trim: true,
    };
    
    let result = validate_input(content, options);
    
    if result.is_valid {
        Ok(result.sanitized_value.unwrap_or_else(|| content.to_string()))
    } else {
        Err(ValidationError { message: result.message })
    }
}

/// 验证会话标题
pub fn validate_conversation_title(title: &str) -> Result<String, ValidationError> {
    let options = ValidationOptions {
        max_length: 200,
        min_length: 1,
        allow_empty: false,
        allow_special_chars: false,
        allow_html: false,
        trim: true,
    };
    
    let result = validate_input(title, options);
    
    if result.is_valid {
        Ok(result.sanitized_value.unwrap_or_else(|| title.to_string()))
    } else {
        Err(ValidationError { message: result.message })
    }
}

/// 验证搜索关键词
pub fn validate_search_keyword(keyword: &str) -> Result<String, ValidationError> {
    let options = ValidationOptions {
        max_length: 100,
        min_length: 1,
        allow_empty: false,
        allow_special_chars: true,
        allow_html: false,
        trim: true,
    };
    
    let result = validate_input(keyword, options);
    
    if result.is_valid {
        Ok(result.sanitized_value.unwrap_or_else(|| keyword.to_string()))
    } else {
        Err(ValidationError { message: result.message })
    }
}

/// 转义HTML特殊字符
pub fn escape_html(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    
    for c in text.chars() {
        match c {
            '&' => result.push_str("&amp;"),
            '<' => result.push_str("&lt;"),
            '>' => result.push_str("&gt;"),
            '"' => result.push_str("&quot;"),
            '\'' => result.push_str("&#39;"),
            '/' => result.push_str("&#x2F;"),
            _ => result.push(c),
        }
    }
    
    result
}

/// 清理和规范化输入
pub fn sanitize_input(input: &str) -> String {
    let mut sanitized = input.to_string();
    
    // 移除控制字符
    sanitized = sanitized.chars()
        .filter(|c| !c.is_control())
        .collect();
    
    // 移除多余的空白字符
    sanitized = sanitized.replace(r"\s+", " ").trim().to_string();
    
    // 转义HTML特殊字符
    sanitized = escape_html(&sanitized);
    
    sanitized
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_validate_chat_message() {
        // 正常消息
        let result = validate_chat_message("Hello, world!");
        assert!(result.is_ok());
        
        // 空消息
        let result = validate_chat_message("");
        assert!(result.is_err());
        
        // 过长的消息
        let long_message = "a".repeat(6000);
        let result = validate_chat_message(&long_message);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_conversation_title() {
        // 正常标题
        let result = validate_conversation_title("Test Conversation");
        assert!(result.is_ok());
        
        // 包含特殊字符的标题
        let result = validate_conversation_title("Test; Conversation");
        assert!(result.is_err());
    }
    
    #[test]
    fn test_escape_html() {
        let input = "<script>alert('XSS')</script>";
        let escaped = escape_html(input);
        assert_eq!(escaped, "&lt;script&gt;alert(&#39;XSS&#39;)&lt;/script&gt;");
    }
    
    #[test]
    fn test_sanitize_input() {
        let input = "  Hello  <world>  ";
        let sanitized = sanitize_input(input);
        assert_eq!(sanitized, "Hello &lt;world&gt;");
    }
}
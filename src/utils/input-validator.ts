/**
 * 输入验证工具
 * 提供用户输入的安全验证和过滤功能
 */

/**
 * 输入验证结果
 */
export interface ValidationResult {
  isValid: boolean;
  message: string;
  sanitizedValue?: string;
}

/**
 * 验证配置选项
 */
export interface ValidationOptions {
  maxLength?: number;
  minLength?: number;
  allowEmpty?: boolean;
  allowSpecialChars?: boolean;
  allowHtml?: boolean;
  trim?: boolean;
}

/**
 * 默认验证配置
 */
const DEFAULT_OPTIONS: ValidationOptions = {
  maxLength: 10000,
  minLength: 1,
  allowEmpty: false,
  allowSpecialChars: true,
  allowHtml: false,
  trim: true
};

/**
 * 特殊字符黑名单
 */
const SPECIAL_CHARS_BLACKLIST = [
  // 潜在的SQL注入字符
  ';', '--', '/*', '*/', 'xp_', 'sp_',
  // 潜在的XSS攻击字符
  '<script', 'javascript:', 'onload', 'onerror', 'onclick'
];

/**
 * HTML标签黑名单
 */
const HTML_TAGS_BLACKLIST = [
  'script', 'iframe', 'object', 'embed', 'form', 'input',
  'button', 'select', 'textarea', 'meta', 'link', 'style'
];

/**
 * 验证用户输入
 * @param input 输入内容
 * @param options 验证选项
 * @returns 验证结果
 */
export function validateInput(input: string, options: ValidationOptions = {}): ValidationResult {
  const config = { ...DEFAULT_OPTIONS, ...options };
  
  // 预处理输入
  let sanitizedValue = input;
  if (config.trim) {
    sanitizedValue = sanitizedValue.trim();
  }
  
  // 检查空值
  if (!config.allowEmpty && sanitizedValue.length === 0) {
    return {
      isValid: false,
      message: '输入内容不能为空'
    };
  }
  
  // 检查最小长度
  if (sanitizedValue.length < config.minLength!) {
    return {
      isValid: false,
      message: `输入内容至少需要${config.minLength}个字符`
    };
  }
  
  // 检查最大长度
  if (sanitizedValue.length > config.maxLength!) {
    return {
      isValid: false,
      message: `输入内容不能超过${config.maxLength}个字符`
    };
  }
  
  // 检查特殊字符
  if (!config.allowSpecialChars) {
    const hasSpecialChars = SPECIAL_CHARS_BLACKLIST.some(char => 
      sanitizedValue.toLowerCase().includes(char.toLowerCase())
    );
    
    if (hasSpecialChars) {
      return {
        isValid: false,
        message: '输入内容包含不允许的特殊字符'
      };
    }
  }
  
  // 检查HTML标签
  if (!config.allowHtml) {
    const hasHtmlTags = HTML_TAGS_BLACKLIST.some(tag => {
      const pattern = new RegExp(`<${tag}[^>]*>|<\\/${tag}>`, 'gi');
      return pattern.test(sanitizedValue);
    });
    
    if (hasHtmlTags) {
      // 移除HTML标签
      sanitizedValue = sanitizedValue.replace(/<[^>]*>/g, '');
      
      // 如果移除了标签，检查是否还有有效内容
      if (sanitizedValue.trim().length === 0) {
        return {
          isValid: false,
          message: '输入内容包含不安全的HTML标签'
        };
      }
    }
  }
  
  // 额外的安全检查：URL和协议
  const urlPattern = /(https?:\/\/|ftp:\/\/|file:\/\/)/gi;
  if (urlPattern.test(sanitizedValue)) {
    // 检查是否为潜在的危险URL
    const dangerousProtocols = ['file:', 'javascript:', 'vbscript:'];
    const hasDangerousUrl = dangerousProtocols.some(protocol => 
      sanitizedValue.toLowerCase().includes(protocol)
    );
    
    if (hasDangerousUrl) {
      return {
        isValid: false,
        message: '输入内容包含不安全的URL协议'
      };
    }
  }
  
  return {
    isValid: true,
    message: '输入验证通过',
    sanitizedValue
  };
}

/**
 * 验证聊天消息输入
 * @param message 消息内容
 * @returns 验证结果
 */
export function validateChatMessage(message: string): ValidationResult {
  return validateInput(message, {
    maxLength: 5000,
    minLength: 1,
    allowEmpty: false,
    allowSpecialChars: true,
    allowHtml: false,
    trim: true
  });
}

/**
 * 验证会话标题
 * @param title 标题内容
 * @returns 验证结果
 */
export function validateConversationTitle(title: string): ValidationResult {
  return validateInput(title, {
    maxLength: 200,
    minLength: 1,
    allowEmpty: false,
    allowSpecialChars: false,
    allowHtml: false,
    trim: true
  });
}

/**
 * 验证搜索关键词
 * @param keyword 搜索关键词
 * @returns 验证结果
 */
export function validateSearchKeyword(keyword: string): ValidationResult {
  return validateInput(keyword, {
    maxLength: 100,
    minLength: 1,
    allowEmpty: false,
    allowSpecialChars: true,
    allowHtml: false,
    trim: true
  });
}

/**
 * 转义HTML特殊字符
 * @param text 要转义的文本
 * @returns 转义后的文本
 */
export function escapeHtml(text: string): string {
  const htmlEntities: Record<string, string> = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    "'": '&#39;',
    '/': '&#x2F;'
  };
  
  return text.replace(/[&<>"'\/]/g, char => htmlEntities[char]);
}

/**
 * 清理和规范化输入
 * @param input 输入内容
 * @returns 清理后的内容
 */
export function sanitizeInput(input: string): string {
  // 移除控制字符
  let sanitized = input.replace(/[\x00-\x1F\x7F]/g, '');
  
  // 移除多余的空白字符
  sanitized = sanitized.replace(/\s+/g, ' ').trim();
  
  // 转义HTML特殊字符
  sanitized = escapeHtml(sanitized);
  
  return sanitized;
}
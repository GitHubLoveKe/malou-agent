# 全面测试和优化方案

## 测试策略

### 1. 单元测试 (Rust 后端)
```rust
// src-tauri/tests/unit_tests.rs
#[cfg(test)]
mod database_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_document_crud_operations() {
        let db = setup_test_database().await;
        
        // 测试创建文档
        let doc = DocumentCreate {
            title: "测试文档".to_string(),
            content: "测试内容".to_string(),
            embedding: Some(vec![0.1, 0.2, 0.3]),
            metadata: None,
        };
        
        let created = db.create_document(doc).await.unwrap();
        assert!(!created.id.is_empty());
        
        // 测试查询文档
        let retrieved = db.get_document(&created.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().title, "测试文档");
        
        // 测试更新文档
        let update = DocumentUpdate {
            title: Some("更新后的标题".to_string()),
            content: None,
            embedding: None,
            metadata: None,
        };
        
        let updated = db.update_document(&created.id, update).await.unwrap();
        assert_eq!(updated.unwrap().title, "更新后的标题");
        
        // 测试删除文档
        let deleted = db.delete_document(&created.id).await.unwrap();
        assert!(deleted);
    }
}

#[cfg(test)]
mod api_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_send_message_command() {
        let request = MessageRequest {
            content: "Hello, world!".to_string(),
            conversation_id: None,
        };
        
        let response = send_message(request).await.unwrap();
        assert!(response.content.contains("Hello, world!"));
        assert!(!response.conversation_id.is_empty());
    }
}
```

### 2. 集成测试
```typescript
// src/tests/integration.test.ts
import { sendMessage, createDocument } from '@/api/tauri-api';

describe('Integration Tests', () => {
  test('should handle message sending and response', async () => {
    const response = await sendMessage({
      content: 'Test message',
      conversationId: 'test-conversation'
    });
    
    expect(response.content).toContain('Test message');
    expect(response.conversationId).toBe('test-conversation');
  });

  test('should manage document lifecycle', async () => {
    const doc = await createDocument({
      title: 'Test Document',
      content: 'Test content for integration test'
    });
    
    expect(doc.id).toBeTruthy();
    expect(doc.title).toBe('Test Document');
  });
});
```

## 性能优化

### 1. 数据库优化
```rust
// src-tauri/src/database.rs
impl SQLiteDatabase {
    pub async fn optimize_performance(&self) -> Result<(), DatabaseError> {
        let conn = self.conn.lock().await;
        
        tokio::task::spawn_blocking(move || {
            // 启用 WAL 模式
            conn.execute_batch("
                PRAGMA journal_mode=WAL;
                PRAGMA synchronous=NORMAL;
                PRAGMA cache_size=10000;
                PRAGMA temp_store=memory;
            ")?;
            
            // 创建性能索引
            conn.execute_batch("
                CREATE INDEX IF NOT EXISTS idx_documents_content_fts 
                ON documents(content);
                
                CREATE INDEX IF NOT EXISTS idx_documents_updated_recent 
                ON documents(updated_at DESC)
                WHERE updated_at > datetime('now', '-7 days');
            ")?;
            
            Ok(())
        }).await?
    }
}
```

### 2. 内存管理优化
```typescript
// src/composables/useMemoryOptimization.ts
export function useMemoryOptimization() {
  // 虚拟滚动实现
  const virtualScroll = (container: HTMLElement, items: any[]) => {
    // 实现虚拟滚动逻辑
  };
  
  // 图片懒加载
  const lazyLoadImages = () => {
    const images = document.querySelectorAll('img[data-src]');
    const imageObserver = new IntersectionObserver((entries) => {
      entries.forEach(entry => {
        if (entry.isIntersecting) {
          const img = entry.target as HTMLImageElement;
          img.src = img.dataset.src!;
          img.classList.remove('lazy');
          imageObserver.unobserve(img);
        }
      });
    });
    
    images.forEach(img => imageObserver.observe(img));
  };
  
  return { virtualScroll, lazyLoadImages };
}
```

## 错误处理和监控

### 1. 全局错误处理
```typescript
// src/utils/errorHandler.ts
class ErrorHandler {
  static handle(error: Error, context: string) {
    console.error(`[${context}] Error:`, error);
    
    // 记录错误到日志
    this.logError(error, context);
    
    // 显示用户友好的错误信息
    ElMessage.error(this.getUserFriendlyMessage(error));
    
    // 上报错误到监控系统
    this.reportError(error, context);
  }
  
  private static getUserFriendlyMessage(error: Error): string {
    if (error.message.includes('network')) {
      return '网络连接出现问题，请检查网络设置';
    }
    if (error.message.includes('database')) {
      return '数据库操作失败，请稍后重试';
    }
    return '操作失败，请重试';
  }
}

// 全局错误捕获
window.addEventListener('error', (event) => {
  ErrorHandler.handle(event.error, 'Global Error');
});

window.addEventListener('unhandledrejection', (event) => {
  ErrorHandler.handle(event.reason, 'Unhandled Promise Rejection');
});
```

### 2. 性能监控
```typescript
// src/utils/performanceMonitor.ts
class PerformanceMonitor {
  private static metrics: Map<string, number[]> = new Map();
  
  static startTimer(label: string): string {
    const id = `${label}_${Date.now()}`;
    this.metrics.set(id, [performance.now()]);
    return id;
  }
  
  static endTimer(id: string) {
    const startTime = this.metrics.get(id);
    if (startTime) {
      const duration = performance.now() - startTime[0];
      startTime.push(duration);
      
      // 记录性能指标
      this.recordMetric(id.split('_')[0], duration);
    }
  }
  
  private static recordMetric(operation: string, duration: number) {
    // 发送到分析服务
    console.log(`Performance: ${operation} took ${duration.toFixed(2)}ms`);
  }
  
  static getAverageTiming(operation: string): number {
    // 计算平均耗时
    return 0; // 简化实现
  }
}
```

## 自动化测试脚本

### 1. CI/CD 配置
```yaml
# .github/workflows/test.yml
name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: windows-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        
    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        
    - name: Install dependencies
      run: |
        npm ci
        cd src-tauri && cargo build --verbose
        
    - name: Run frontend tests
      run: npm test
      
    - name: Run backend tests
      run: cd src-tauri && cargo test
      
    - name: Run integration tests
      run: npm run test:integration
      
    - name: Run performance tests
      run: npm run test:performance
```

### 2. 测试覆盖率报告
```json
// jest.config.js
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'jsdom',
  collectCoverage: true,
  coverageDirectory: 'coverage',
  coverageReporters: ['html', 'text', 'lcov'],
  collectCoverageFrom: [
    'src/**/*.{ts,vue}',
    '!src/main.ts',
    '!src/router/index.ts'
  ]
};
```

## 负载测试

```typescript
// src/tests/load.test.ts
import { sendMessage } from '@/api/tauri-api';

describe('Load Testing', () => {
  test('should handle concurrent requests', async () => {
    const promises = Array(100).fill(null).map((_, i) => 
      sendMessage({
        content: `Test message ${i}`,
        conversationId: `conversation-${i}`
      })
    );
    
    const results = await Promise.all(promises);
    expect(results).toHaveLength(100);
    
    // 验证所有请求都成功处理
    results.forEach((result, index) => {
      expect(result.content).toContain(`Test message ${index}`);
    });
  }, 30000); // 30秒超时
});
```

## 用户体验优化

### 1. 加载状态管理
```vue
<!-- src/components/common/LoadingSpinner.vue -->
<template>
  <div class="loading-spinner" :class="{ 'full-screen': fullscreen }">
    <div class="spinner"></div>
    <p v-if="message" class="loading-message">{{ message }}</p>
  </div>
</template>

<script setup lang="ts">
interface Props {
  fullscreen?: boolean;
  message?: string;
}

const props = withDefaults(defineProps<Props>(), {
  fullscreen: false,
  message: '加载中...'
});
</script>
```

### 2. 缓存策略
```typescript
// src/utils/cacheManager.ts
class CacheManager {
  private static cache = new Map<string, { data: any; timestamp: number }>();
  private static readonly TTL = 5 * 60 * 1000; // 5分钟
  
  static set(key: string, data: any) {
    this.cache.set(key, {
      data,
      timestamp: Date.now()
    });
  }
  
  static get(key: string): any {
    const item = this.cache.get(key);
    if (!item) return null;
    
    // 检查是否过期
    if (Date.now() - item.timestamp > this.TTL) {
      this.cache.delete(key);
      return null;
    }
    
    return item.data;
  }
  
  static clearExpired() {
    const now = Date.now();
    for (const [key, item] of this.cache.entries()) {
      if (now - item.timestamp > this.TTL) {
        this.cache.delete(key);
      }
    }
  }
}
```

这些测试和优化措施将确保应用的稳定性、性能和用户体验达到最佳水平。
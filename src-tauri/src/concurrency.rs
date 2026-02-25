use tokio::sync::{Semaphore, RwLock};
use std::collections::HashMap;
use std::sync::Arc;

/// 资源配额管理器
pub struct ResourceManager {
    // 不同类型资源的信号量
    cpu_permits: Semaphore,
    io_permits: Semaphore,
    memory_permits: Semaphore,
    // 特定资源的锁
    resource_locks: RwLock<HashMap<String, Arc<Semaphore>>>,
}

impl ResourceManager {
    pub fn new(cpu_limit: usize, io_limit: usize, memory_limit: usize) -> Self {
        Self {
            cpu_permits: Semaphore::new(cpu_limit),
            io_permits: Semaphore::new(io_limit),
            memory_permits: Semaphore::new(memory_limit),
            resource_locks: RwLock::new(HashMap::new()),
        }
    }

    /// 获取CPU资源许可
    pub async fn acquire_cpu_permit(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.cpu_permits.acquire().await.unwrap()
    }

    /// 获取IO资源许可
    pub async fn acquire_io_permit(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.io_permits.acquire().await.unwrap()
    }

    /// 获取特定资源的互斥锁
    pub async fn acquire_resource_lock(&self, resource_id: &str, max_concurrent: usize) -> tokio::sync::SemaphorePermit<'_> {
        let semaphore = {
            let locks = self.resource_locks.read().await;
            if let Some(sem) = locks.get(resource_id) {
                sem.clone()
            } else {
                drop(locks);
                let mut locks = self.resource_locks.write().await;
                locks.entry(resource_id.to_string())
                    .or_insert_with(|| Arc::new(Semaphore::new(max_concurrent)))
                    .clone()
            }
        };

        semaphore.acquire().await.unwrap()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn test_allocate() {
        let pool = MemoryPool::new(32, 10); // 每个块32字节，初始分配10个块
        let ptr = pool.allocate();
        assert!(!ptr.as_ptr().is_null()); // 确保分配的内存非空
    }

    #[test]
    fn test_deallocate_and_reuse() {
        let pool = MemoryPool::new(32, 10);
        let ptr1 = pool.allocate();
        let ptr2 = pool.allocate();
        
        assert!(ptr1 != ptr2); // 确保分配的两个内存地址不同
        
        pool.deallocate(ptr1);
        let ptr3 = pool.allocate();
        
        assert_eq!(ptr1, ptr3); // 确保释放的内存块可以被重新分配
    }

    #[test]
    fn test_multiple_allocations() {
        let pool = MemoryPool::new(32, 5); // 每个块32字节，初始分配5个块
        let mut allocated = Vec::new();

        for _ in 0..5 {
            let ptr = pool.allocate();
            allocated.push(ptr);
        }

        // 所有初始块应该已分配完
        assert!(allocated.len() == 5);

        // 释放一个块
        pool.deallocate(allocated.pop().unwrap());

        // 再次分配一个块，应该不会触发新的内存分配
        let ptr = pool.allocate();
        assert!(allocated.iter().all(|&p| p != ptr));
    }

    #[test]
    fn test_expand_pool() {
        let pool = MemoryPool::new(32, 2); // 每个块32字节，初始分配2个块
        let ptr1 = pool.allocate();
        let ptr2 = pool.allocate();

        // 初始块已分配完，再次分配应该触发池扩展
        let ptr3 = pool.allocate();
        assert!(!ptr3.as_ptr().is_null()); // 确保分配的内存非空
        assert!(ptr1 != ptr3 && ptr2 != ptr3); // 确保新分配的块不同
    }
}

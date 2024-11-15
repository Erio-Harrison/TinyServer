use std::ptr::NonNull;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::mem::align_of;

#[test]
fn test_memory_pool_basic() {
    let pool = MemoryPool::new(64, 10);
    
    // 分配一个块
    let ptr1 = pool.allocate();
    
    // 确保指针非空且对齐正确
    assert!(!ptr1.as_ptr().is_null());
    assert_eq!(ptr1.as_ptr() as usize % align_of::<usize>(), 0);
    
    // 分配第二个块
    let ptr2 = pool.allocate();
    assert!(!ptr2.as_ptr().is_null());
    assert_ne!(ptr1, ptr2);
    
    // 释放并重新分配
    pool.deallocate(ptr1);
    let ptr3 = pool.allocate();
    // 应该得到刚才释放的块
    assert_eq!(ptr1, ptr3);
}

#[test]
fn test_memory_pool_capacity() {
    let initial_blocks = 2;
    let pool = MemoryPool::new(32, initial_blocks);
    
    // 分配所有初始块
    let mut ptrs = Vec::new();
    for _ in 0..initial_blocks {
        ptrs.push(pool.allocate());
    }
    
    // 下一次分配应触发新chunk分配
    let ptr = pool.allocate();
    assert!(!ptr.as_ptr().is_null());
    
    // 清理
    for ptr in ptrs {
        pool.deallocate(ptr);
    }
    pool.deallocate(ptr);
}

#[test]
fn test_memory_pool_concurrent() {
    let pool = Arc::new(MemoryPool::new(128, 50));
    let mut handles = vec![];
    let iterations = 1000;
    
    for _ in 0..8 {
        let pool_clone = pool.clone();
        let handle = thread::spawn(move || {
            let mut ptrs = Vec::new();
            for _ in 0..iterations {
                // 分配一些块
                for _ in 0..5 {
                    ptrs.push(pool_clone.allocate());
                }
                // 释放一些块
                for _ in 0..3 {
                    if let Some(ptr) = ptrs.pop() {
                        pool_clone.deallocate(ptr);
                    }
                }
                thread::sleep(Duration::from_micros(1));
            }
            // 清理剩余的块
            for ptr in ptrs {
                pool_clone.deallocate(ptr);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_memory_pool_alignment() {
    let align = align_of::<usize>();
    let pool = MemoryPool::new(align * 4, 5);
    
    // 检查多个分配的对齐情况
    for _ in 0..10 {
        let ptr = pool.allocate();
        assert_eq!(ptr.as_ptr() as usize % align, 0);
        pool.deallocate(ptr);
    }
}

#[test]
fn test_memory_pool_stress() {
    let pool = Arc::new(MemoryPool::new(256, 10));
    let mut handles = vec![];
    
    // 创建多个线程，反复进行分配和释放
    for _ in 0..16 {
        let pool_clone = pool.clone();
        let handle = thread::spawn(move || {
            let mut ptrs = Vec::new();
            for _ in 0..100 {
                match rand::random::<bool>() {
                    true => {
                        ptrs.push(pool_clone.allocate());
                    }
                    false => {
                        if let Some(ptr) = ptrs.pop() {
                            pool_clone.deallocate(ptr);
                        }
                    }
                }
            }
            // 清理剩余的分配
            for ptr in ptrs {
                pool_clone.deallocate(ptr);
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_memory_pool_different_sizes() {
    // 测试不同大小的块
    let sizes = [16, 32, 64, 128, 256, 512, 1024];
    for &size in &sizes {
        let pool = MemoryPool::new(size, 5);
        let mut ptrs = Vec::new();
        
        // 分配一些块
        for _ in 0..10 {
            ptrs.push(pool.allocate());
        }
        
        // 验证所有指针都是有效的且不重叠
        for i in 0..ptrs.len() {
            for j in i+1..ptrs.len() {
                let start1 = ptrs[i].as_ptr() as usize;
                let end1 = start1 + size;
                let start2 = ptrs[j].as_ptr() as usize;
                let end2 = start2 + size;
                assert!(end1 <= start2 || end2 <= start1);
            }
        }
        
        // 清理
        for ptr in ptrs {
            pool.deallocate(ptr);
        }
    }
}

#[test]
fn test_memory_pool_reuse() {
    let pool = MemoryPool::new(64, 5);
    let mut used_ptrs = Vec::new();
    let mut freed_ptrs = Vec::new();
    
    // 分配一些块
    for _ in 0..10 {
        used_ptrs.push(pool.allocate());
    }
    
    // 释放一半的块
    for _ in 0..5 {
        if let Some(ptr) = used_ptrs.pop() {
            freed_ptrs.push(ptr);
            pool.deallocate(ptr);
        }
    }
    
    // 重新分配，应该得到之前释放的块
    for expected_ptr in freed_ptrs {
        let new_ptr = pool.allocate();
        assert_eq!(expected_ptr, new_ptr);
    }
    
    // 清理剩余的块
    for ptr in used_ptrs {
        pool.deallocate(ptr);
    }
}

#[test]
#[should_panic]
fn test_memory_pool_double_free() {
    let pool = MemoryPool::new(32, 5);
    let ptr = pool.allocate();
    pool.deallocate(ptr);
    // 这应该会导致panic
    pool.deallocate(ptr);
}
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::NonNull;
use std::sync::Mutex;

pub struct MemoryPool {
    block_size: usize,
    free_list: Mutex<Option<NonNull<Block>>>,
    chunks: Mutex<Vec<NonNull<u8>>>,
}

struct Block {
    next: Option<NonNull<Block>>,
}

impl MemoryPool {
    pub fn new(block_size: usize, initial_blocks: usize) -> Self {
        let pool = MemoryPool {
            block_size: block_size.max(std::mem::size_of::<Block>()),
            free_list: Mutex::new(None),
            chunks: Mutex::new(Vec::new()),
        };
        pool.allocate_chunk(initial_blocks);
        pool
    }

    pub fn allocate(&self) -> NonNull<u8> {
        let mut free_list = self.free_list.lock().unwrap();
        if free_list.is_none() {
            drop(free_list);  // 释放锁，因为 allocate_chunk 会再次获取锁
            self.allocate_chunk(100);
            free_list = self.free_list.lock().unwrap();
        }
        let block = free_list.take().unwrap();
        *free_list = unsafe { block.as_ref().next };
        block.cast()
    }

    pub fn deallocate(&self, ptr: NonNull<u8>) {
        let block = ptr.cast::<Block>();
        let mut free_list = self.free_list.lock().unwrap();
        unsafe {
            (*block.as_ptr()).next = *free_list;
        }
        *free_list = Some(block);
    }

    fn allocate_chunk(&self, blocks: usize) {
        let layout = Layout::array::<u8>(self.block_size * blocks).unwrap();
        let chunk = unsafe { NonNull::new(alloc(layout)).unwrap() };
        
        let mut free_list = self.free_list.lock().unwrap();
        for i in 0..blocks {
            let block = unsafe { chunk.as_ptr().add(i * self.block_size) as *mut Block };
            let block = NonNull::new(block).unwrap();
            unsafe {
                (*block.as_ptr()).next = *free_list;
            }
            *free_list = Some(block);
        }

        self.chunks.lock().unwrap().push(chunk);
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        let chunks = self.chunks.get_mut().unwrap();
        for &chunk in chunks.iter() {
            unsafe {
                dealloc(chunk.as_ptr(), Layout::array::<u8>(self.block_size * 100).unwrap());
            }
        }
    }
}
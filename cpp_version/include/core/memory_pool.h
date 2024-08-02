#pragma once

#include <vector>
#include <cstddef>
#include <mutex>

class MemoryPool {
public:
    MemoryPool(size_t block_size, size_t initial_blocks = 100);
    ~MemoryPool();

    void* allocate();
    void deallocate(void* ptr);

private:
    struct Block {
        Block* next;
    };

    size_t block_size_;
    Block* free_list_;
    std::vector<char*> chunks_;
    size_t blocks_per_chunk_;
    std::mutex mutex_;

    void allocate_chunk();
};
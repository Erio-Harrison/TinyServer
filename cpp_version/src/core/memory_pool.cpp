#include "core/memory_pool.h"
#include <cstdlib>
#include <algorithm>

MemoryPool::MemoryPool(size_t block_size, size_t initial_blocks)
    : block_size_(std::max(block_size, sizeof(Block))), 
      blocks_per_chunk_(initial_blocks),free_list_(nullptr) 
{
    allocate_chunk();
}

MemoryPool::~MemoryPool() {
    for (auto chunk : chunks_) {
        std::free(chunk);
    }
}

void* MemoryPool::allocate() {
    std::lock_guard<std::mutex> lock(mutex_);
    if (free_list_ == nullptr) {
        allocate_chunk();
    }

    Block* block = free_list_;
    free_list_ = block->next;
    return block;
}

void MemoryPool::deallocate(void* ptr) {
    std::lock_guard<std::mutex> lock(mutex_);
    Block* block = static_cast<Block*>(ptr);
    block->next = free_list_;
    free_list_ = block;
}

void MemoryPool::allocate_chunk(){
    size_t chunk_size = block_size_ * blocks_per_chunk_;
    char* chunk = static_cast<char*>(std::malloc(chunk_size));
    chunks_.push_back(chunk);

    for(size_t i = 0; i< chunk_size; i+=block_size_){
        Block* block = reinterpret_cast<Block*>(chunk + i);
        block->next = free_list_;
        free_list_ = block;
    }
}
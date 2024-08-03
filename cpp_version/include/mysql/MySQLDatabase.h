#pragma once

#include <string>
#include <memory>
#include <mysql/mysql.h>
#include "core/connection_pool.h"
#include "core/memory_pool.h"

class MySQLConnection : public Connection {
public:
    MySQLConnection(const std::string& host, const std::string& user, const std::string& password, const std::string& database);
    ~MySQLConnection() override;
    MYSQL* get_mysql();

private:
    MYSQL* mysql_;
};

class MySQLDatabaseManager {
public:
    MySQLDatabaseManager(const std::string& host, const std::string& user, const std::string& password, const std::string& database, size_t max_connections);
    
    std::unique_ptr<MySQLConnection> get_connection();
    void release_connection(std::unique_ptr<MySQLConnection> conn);
    void* allocate_memory();
    void deallocate_memory(void* ptr);
    bool execute_query(const std::string& query);

private:
    ConnectionPool connection_pool_;
    MemoryPool memory_pool_;
};
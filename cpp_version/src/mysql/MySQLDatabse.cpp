#include "mysql/MySQLDatabse.h"
#include <stdexcept>

class MySQLConnection : public Connection {
public:
    MySQLConnection(const std::string& host, const std::string& user, const std::string& password, const std::string& database)
        : Connection(-1), mysql_(nullptr) {
        mysql_ = mysql_init(nullptr);
        if (!mysql_) {
            throw std::runtime_error("MySQL init failed");
        }
        if (!mysql_real_connect(mysql_, host.c_str(), user.c_str(), password.c_str(), database.c_str(), 0, nullptr, 0)) {
            throw std::runtime_error(mysql_error(mysql_));
        }
    }

    ~MySQLConnection() override {
        if (mysql_) {
            mysql_close(mysql_);
        }
    }

    MYSQL* get_mysql() { return mysql_; }

private:
    MYSQL* mysql_;
};

class MySQLDatabaseManager {
public:
    MySQLDatabaseManager(const std::string& host, const std::string& user, const std::string& password, const std::string& database, size_t max_connections)
        : connection_pool_(max_connections, [&]() {
              return std::make_unique<MySQLConnection>(host, user, password, database);
          }),
          memory_pool_(1024, 100) // 使用1KB的块大小和初始100个块
    {
    }

    std::unique_ptr<MySQLConnection> get_connection() {
        auto conn = connection_pool_.get_connection();
        return std::unique_ptr<MySQLConnection>(static_cast<MySQLConnection*>(conn.release()));
    }

    void release_connection(std::unique_ptr<MySQLConnection> conn) {
        connection_pool_.release_connection(std::move(conn));
    }

    void* allocate_memory() {
        return memory_pool_.allocate();
    }

    void deallocate_memory(void* ptr) {
        memory_pool_.deallocate(ptr);
    }

    // 示例查询方法
    bool execute_query(const std::string& query) {
        auto conn = get_connection();
        int result = mysql_query(conn->get_mysql(), query.c_str());
        release_connection(std::move(conn));
        return result == 0;
    }

private:
    ConnectionPool connection_pool_;
    MemoryPool memory_pool_;
};
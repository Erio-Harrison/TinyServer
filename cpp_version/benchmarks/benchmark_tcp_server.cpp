#include "network/tcp_server.h"
#include "core/reactor.h"
#include <benchmark/benchmark.h>
#include <thread>
#include <vector>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <unistd.h>
#include <cstring>

static void BM_TcpServerThroughput(benchmark::State& state) {
    Reactor reactor;
    TcpServer server(reactor, "127.0.0.1", 8080);

    std::atomic<size_t> total_bytes_received(0);

    server.set_receive_handler([&](int client_fd, const char* data, size_t len) {
        total_bytes_received += len;
    });

    server.start();

    std::thread server_thread([&reactor]() {
        reactor.run();
    });

    // 给服务器一些时间来启动
    std::this_thread::sleep_for(std::chrono::milliseconds(100));

    std::vector<int> client_fds;
    for (int i = 0; i < state.range(0); ++i) {
        int client_fd = socket(AF_INET, SOCK_STREAM, 0);
        sockaddr_in addr;
        addr.sin_family = AF_INET;
        addr.sin_port = htons(8080);
        inet_pton(AF_INET, "127.0.0.1", &addr.sin_addr);
        connect(client_fd, (struct sockaddr*)&addr, sizeof(addr));
        client_fds.push_back(client_fd);
    }

    for (auto _ : state) {
        const char* message = "Hello, Server!";
        for (int client_fd : client_fds) {
            send(client_fd, message, strlen(message), 0);
        }
    }

    for (int client_fd : client_fds) {
        close(client_fd);
    }

    reactor.stop();
    server_thread.join();

    state.SetBytesProcessed(total_bytes_received.load());
}

BENCHMARK(BM_TcpServerThroughput)->Arg(1)->Arg(10)->Arg(100);

BENCHMARK_MAIN();
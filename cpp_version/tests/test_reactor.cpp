#include "gtest/gtest.h"
#include "core/reactor.h"
#include <sys/epoll.h>
#include <thread>
#include <chrono>
#include <unistd.h>

TEST(ReactorTest, BasicFunctionality) {
    Reactor reactor;
    int pipe_fds[2];
    ASSERT_EQ(pipe(pipe_fds), 0);

    bool handler_called = false;
    reactor.add_handler(pipe_fds[0], EPOLLIN, [&handler_called, &reactor, &pipe_fds](uint32_t events) {
        if (events & EPOLLIN) {
            char buf[1];
            read(pipe_fds[0], buf, 1);
            handler_called = true;
            reactor.stop();
        }
    });

    std::thread t([&reactor]() {
        reactor.run();
    });

    // 给反应器一些时间来启动
    std::this_thread::sleep_for(std::chrono::milliseconds(100));

    write(pipe_fds[1], "x", 1);

    t.join();

    EXPECT_TRUE(handler_called);

    close(pipe_fds[0]);
    close(pipe_fds[1]);
}
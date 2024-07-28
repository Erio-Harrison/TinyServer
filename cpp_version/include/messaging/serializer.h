#pragma once

#include <vector>
#include <string>
#include <cstdint>
#include <stdexcept>

class Serializer {
public:
    Serializer() = default;

    // 基本类型的序列化
    void write(bool value);
    void write(int32_t value);
    void write(int64_t value);
    void write(float value);
    void write(double value);
    void write(const std::string& value);

    // 获取序列化后的数据
    const std::vector<uint8_t>& data() const { return buffer_; }

private:
    std::vector<uint8_t> buffer_;
};

class Deserializer {
public:
    Deserializer(const std::vector<uint8_t>& data);

    // 基本类型的反序列化
    bool read_bool();
    int32_t read_int32();
    int64_t read_int64();
    float read_float();
    double read_double();
    std::string read_string();

    // 检查是否还有数据可读
    bool has_more() const { return position_ < data_.size(); }

private:
    const std::vector<uint8_t>& data_;
    size_t position_ = 0;

    template<typename T>
    T read_raw();
};
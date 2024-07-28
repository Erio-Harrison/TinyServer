#include "messaging/serializer.h"
#include <cstring>
#include <algorithm>

// Serializer implementation

void Serializer::write(bool value) {
    buffer_.push_back(value ? 1 : 0);
}

void Serializer::write(int32_t value) {
    for (int i = 0; i < 4; ++i) {
        buffer_.push_back((value >> (i * 8)) & 0xFF);
    }
}

void Serializer::write(int64_t value) {
    for (int i = 0; i < 8; ++i) {
        buffer_.push_back((value >> (i * 8)) & 0xFF);
    }
}

void Serializer::write(float value) {
    uint32_t raw;
    std::memcpy(&raw, &value, sizeof(float));
    write(static_cast<int32_t>(raw));
}

void Serializer::write(double value) {
    uint64_t raw;
    std::memcpy(&raw, &value, sizeof(double));
    write(static_cast<int64_t>(raw));
}

void Serializer::write(const std::string& value) {
    write(static_cast<int32_t>(value.size()));
    buffer_.insert(buffer_.end(), value.begin(), value.end());
}

// Deserializer implementation

Deserializer::Deserializer(const std::vector<uint8_t>& data) : data_(data) {}

bool Deserializer::read_bool() {
    if (position_ >= data_.size()) {
        throw std::out_of_range("End of buffer reached");
    }
    return data_[position_++] != 0;
}

int32_t Deserializer::read_int32() {
    return read_raw<int32_t>();
}

int64_t Deserializer::read_int64() {
    return read_raw<int64_t>();
}

float Deserializer::read_float() {
    uint32_t raw = read_raw<uint32_t>();
    float value;
    std::memcpy(&value, &raw, sizeof(float));
    return value;
}

double Deserializer::read_double() {
    uint64_t raw = read_raw<uint64_t>();
    double value;
    std::memcpy(&value, &raw, sizeof(double));
    return value;
}

std::string Deserializer::read_string() {
    int32_t length = read_int32();
    if (position_ + length > data_.size()) {
        throw std::out_of_range("String length exceeds buffer size");
    }
    std::string result(data_.begin() + position_, data_.begin() + position_ + length);
    position_ += length;
    return result;
}

template<typename T>
T Deserializer::read_raw() {
    if (position_ + sizeof(T) > data_.size()) {
        throw std::out_of_range("End of buffer reached");
    }
    T result = 0;
    for (size_t i = 0; i < sizeof(T); ++i) {
        result |= static_cast<T>(data_[position_++]) << (i * 8);
    }
    return result;
}
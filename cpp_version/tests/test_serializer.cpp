#include "gtest/gtest.h"
#include "messaging/serializer.h"
#include <limits>
#include <cmath>

class SerializerTest : public ::testing::Test {
protected:
    Serializer serializer;
    Deserializer* deserializer;

    void SetUp() override {
        serializer = Serializer();
    }

    void TearDown() override {
        delete deserializer;
    }

    void createDeserializer() {
        deserializer = new Deserializer(serializer.data());
    }
};

TEST_F(SerializerTest, BoolSerialization) {
    serializer.write(true);
    serializer.write(false);
    createDeserializer();

    EXPECT_TRUE(deserializer->read_bool());
    EXPECT_FALSE(deserializer->read_bool());
}

TEST_F(SerializerTest, Int32Serialization) {
    serializer.write(42);
    serializer.write(-17);
    serializer.write(std::numeric_limits<int32_t>::max());
    serializer.write(std::numeric_limits<int32_t>::min());
    createDeserializer();

    EXPECT_EQ(deserializer->read_int32(), 42);
    EXPECT_EQ(deserializer->read_int32(), -17);
    EXPECT_EQ(deserializer->read_int32(), std::numeric_limits<int32_t>::max());
    EXPECT_EQ(deserializer->read_int32(), std::numeric_limits<int32_t>::min());
}

TEST_F(SerializerTest, FloatSerialization) {
    serializer.write(3.14159f);
    serializer.write(-2.71828f);
    serializer.write(std::numeric_limits<float>::max());
    serializer.write(std::numeric_limits<float>::min());
    createDeserializer();

    EXPECT_FLOAT_EQ(deserializer->read_float(), 3.14159f);
    EXPECT_FLOAT_EQ(deserializer->read_float(), -2.71828f);
    EXPECT_FLOAT_EQ(deserializer->read_float(), std::numeric_limits<float>::max());
    EXPECT_FLOAT_EQ(deserializer->read_float(), std::numeric_limits<float>::min());
}

TEST_F(SerializerTest, DoubleSerialization) {
    serializer.write(3.14159265358979323846);
    serializer.write(-2.71828182845904523536);
    serializer.write(std::numeric_limits<double>::max());
    serializer.write(std::numeric_limits<double>::min());
    createDeserializer();

    EXPECT_DOUBLE_EQ(deserializer->read_double(), 3.14159265358979323846);
    EXPECT_DOUBLE_EQ(deserializer->read_double(), -2.71828182845904523536);
    EXPECT_DOUBLE_EQ(deserializer->read_double(), std::numeric_limits<double>::max());
    EXPECT_DOUBLE_EQ(deserializer->read_double(), std::numeric_limits<double>::min());
}

TEST_F(SerializerTest, StringSerialization) {
    serializer.write(std::string("Hello, World!"));
    serializer.write(std::string(""));
    serializer.write(std::string("Unicode: ñ, é, 汉字"));
    createDeserializer();

    EXPECT_EQ(deserializer->read_string(), "Hello, World!");
    EXPECT_EQ(deserializer->read_string(), "");
    EXPECT_EQ(deserializer->read_string(), "Unicode: ñ, é, 汉字");
}

TEST_F(SerializerTest, MixedTypeSerialization) {
    serializer.write(true);
    serializer.write(42);
    serializer.write(3.14159f);
    serializer.write(std::string("Mixed types"));
    createDeserializer();

    EXPECT_TRUE(deserializer->read_bool());
    EXPECT_EQ(deserializer->read_int32(), 42);
    EXPECT_FLOAT_EQ(deserializer->read_float(), 3.14159f);
    EXPECT_EQ(deserializer->read_string(), "Mixed types");
}

TEST_F(SerializerTest, OutOfRangeException) {
    serializer.write(42);
    createDeserializer();

    EXPECT_EQ(deserializer->read_int32(), 42);
    EXPECT_THROW(deserializer->read_bool(), std::out_of_range);
}

TEST_F(SerializerTest, LargeStringSerialization) {
    std::string large_string(1000000, 'a');
    serializer.write(large_string);
    createDeserializer();

    EXPECT_EQ(deserializer->read_string(), large_string);
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_deserialize_i32() {
        let value: i32 = 12345;
        let mut serializer = Serializer::new();
        value.serialize(&mut serializer).expect("Failed to serialize i32");

        let data = serializer.data();
        let mut deserializer = Deserializer::new(data);
        let deserialized_value = i32::deserialize(&mut deserializer).expect("Failed to deserialize i32");

        assert_eq!(value, deserialized_value);
    }

    #[test]
    fn test_serialize_deserialize_f64() {
        let value: f64 = 12345.6789;
        let mut serializer = Serializer::new();
        value.serialize(&mut serializer).expect("Failed to serialize f64");

        let data = serializer.data();
        let mut deserializer = Deserializer::new(data);
        let deserialized_value = f64::deserialize(&mut deserializer).expect("Failed to deserialize f64");

        assert_eq!(value, deserialized_value);
    }

    #[test]
    fn test_serialize_deserialize_bool() {
        let value: bool = true;
        let mut serializer = Serializer::new();
        value.serialize(&mut serializer).expect("Failed to serialize bool");

        let data = serializer.data();
        let mut deserializer = Deserializer::new(data);
        let deserialized_value = bool::deserialize(&mut deserializer).expect("Failed to deserialize bool");

        assert_eq!(value, deserialized_value);
    }

    #[test]
    fn test_serialize_deserialize_string() {
        let value = String::from("Hello, world!");
        let mut serializer = Serializer::new();
        value.serialize(&mut serializer).expect("Failed to serialize String");

        let data = serializer.data();
        let mut deserializer = Deserializer::new(data);
        let deserialized_value = String::deserialize(&mut deserializer).expect("Failed to deserialize String");

        assert_eq!(value, deserialized_value);
    }

    #[test]
    fn test_serialize_deserialize_multiple_values() {
        let i32_value: i32 = 42;
        let f64_value: f64 = 3.14159;
        let bool_value: bool = false;
        let string_value = String::from("Test");

        let mut serializer = Serializer::new();
        i32_value.serialize(&mut serializer).expect("Failed to serialize i32");
        f64_value.serialize(&mut serializer).expect("Failed to serialize f64");
        bool_value.serialize(&mut serializer).expect("Failed to serialize bool");
        string_value.serialize(&mut serializer).expect("Failed to serialize String");

        let data = serializer.data();
        let mut deserializer = Deserializer::new(data);

        let deserialized_i32 = i32::deserialize(&mut deserializer).expect("Failed to deserialize i32");
        let deserialized_f64 = f64::deserialize(&mut deserializer).expect("Failed to deserialize f64");
        let deserialized_bool = bool::deserialize(&mut deserializer).expect("Failed to deserialize bool");
        let deserialized_string = String::deserialize(&mut deserializer).expect("Failed to deserialize String");

        assert_eq!(i32_value, deserialized_i32);
        assert_eq!(f64_value, deserialized_f64);
        assert_eq!(bool_value, deserialized_bool);
        assert_eq!(string_value, deserialized_string);
    }

    #[test]
    fn test_serialize_deserialize_empty_string() {
        let value = String::from("");
        let mut serializer = Serializer::new();
        value.serialize(&mut serializer).expect("Failed to serialize empty String");

        let data = serializer.data();
        let mut deserializer = Deserializer::new(data);
        let deserialized_value = String::deserialize(&mut deserializer).expect("Failed to deserialize empty String");

        assert_eq!(value, deserialized_value);
    }
}

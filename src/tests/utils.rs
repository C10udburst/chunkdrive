use std::env;

// This function is used to create a temporary config file for testing purposes
pub fn make_temp_config(encryption: bool, size: usize) -> String {
    if encryption {
        return format!(r#"
buckets:
    local1:
        source:
            type: local
            folder: {}
            max_size: {}
            descriptor_length: 3  # just in case we set extremely small block size for testing
        encryption:
            type: aes
            key: "12345678901234567890123456789012"
        "#, env::temp_dir().display(), size);
    } else {
        return format!(r#"
buckets:
    local2:
        source:
            type: local
            folder: {}
            max_size: {}
            descriptor_length: 3  # just in case we set extremely small block size for testing
        "#, env::temp_dir().display(), size);
    }
}
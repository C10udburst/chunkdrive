use std::env;

pub fn make_temp_config(encryption: bool) -> String {
    if encryption {
        return format!(r#"
buckets:
    local1:
        source:
            type: local
            folder: {}
            max_size: 25
        encryption:
            type: aes
            key: "12345678901234567890123456789012"
        "#, env::temp_dir().display());
    } else {
        return format!(r#"
buckets:
    local2:
        source:
            type: local
            folder: {}
            max_size: 25
        "#, env::temp_dir().display());
    }
}
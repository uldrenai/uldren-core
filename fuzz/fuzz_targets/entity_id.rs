#![no_main]

use libfuzzer_sys::fuzz_target;
use uldren_core_domain::EntityId;

fuzz_target!(|data: &[u8]| {
    let Ok(input) = std::str::from_utf8(data) else {
        return;
    };
    let expected = input.split_once(':').is_some_and(|(kind, slug)| {
        (1..=EntityId::MAX_KIND_LEN).contains(&kind.len())
            && kind.bytes().all(|byte| byte.is_ascii_lowercase())
            && (1..=EntityId::MAX_SLUG_LEN).contains(&slug.len())
            && slug
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            && !slug.starts_with('-')
            && !slug.ends_with('-')
    });
    let result = EntityId::parse(input);
    assert_eq!(result.is_ok(), expected);
    if let Ok(identifier) = result {
        assert_eq!(identifier.as_str(), input);
        assert_eq!(identifier.to_string(), input);
        assert_eq!(EntityId::parse(identifier.as_str()), Ok(identifier));
    }
});

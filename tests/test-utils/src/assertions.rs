//! 测试断言宏
//!
//! 提供便捷的测试断言宏

/// 断言Result是Ok
#[macro_export]
macro_rules! assert_ok {
    ($expr:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("Expected Ok, got Err: {:?}", e),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(v) => v,
            Err(e) => panic!("{}: {:?}", $msg, e),
        }
    };
}

/// 断言Result是Err
#[macro_export]
macro_rules! assert_err {
    ($expr:expr) => {
        match $expr {
            Err(e) => e,
            Ok(v) => panic!("Expected Err, got Ok: {:?}", v),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Err(e) => e,
            Ok(v) => panic!("{}: {:?}", $msg, v),
        }
    };
}

/// 断言Option是Some
#[macro_export]
macro_rules! assert_some {
    ($expr:expr) => {
        match $expr {
            Some(v) => v,
            None => panic!("Expected Some, got None"),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            Some(v) => v,
            None => panic!("{}", $msg),
        }
    };
}

/// 断言Option是None
#[macro_export]
macro_rules! assert_none {
    ($expr:expr) => {
        match $expr {
            None => (),
            Some(v) => panic!("Expected None, got Some: {:?}", v),
        }
    };
    ($expr:expr, $msg:expr) => {
        match $expr {
            None => (),
            Some(v) => panic!("{}: {:?}", $msg, v),
        }
    };
}

/// 断言字符串包含子串
#[macro_export]
macro_rules! assert_contains {
    ($haystack:expr, $needle:expr) => {
        let haystack = &$haystack;
        let needle = &$needle;
        assert!(haystack.contains(needle), "Expected '{}' to contain '{}'", haystack, needle);
    };
}

/// 断言JSON包含字段
#[macro_export]
macro_rules! assert_json_has {
    ($json:expr, $field:expr) => {
        let json = &$json;
        assert!(json.get($field).is_some(), "Expected JSON to have field '{}': {:?}", $field, json);
    };
}

/// 断言JSON字段值相等
#[macro_export]
macro_rules! assert_json_eq {
    ($json:expr, $field:expr, $expected:expr) => {
        let json = &$json;
        let actual = json.get($field).expect(&format!("Field '{}' not found", $field));
        assert_eq!(
            actual, &$expected,
            "JSON field '{}' mismatch: expected {:?}, got {:?}",
            $field, $expected, actual
        );
    };
}

#[cfg(test)]
mod tests {
    use std::result::Result;

    #[test]
    fn test_assert_ok() {
        let result: Result<i32, &str> = Ok(42);
        let value = assert_ok!(result);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_assert_err() {
        let result: Result<i32, &str> = Err("error");
        let error = assert_err!(result);
        assert_eq!(error, "error");
    }

    #[test]
    fn test_assert_some() {
        let option: Option<i32> = Some(42);
        let value = assert_some!(option);
        assert_eq!(value, 42);
    }

    #[test]
    fn test_assert_none() {
        let option: Option<i32> = None;
        assert_none!(option);
    }

    #[test]
    fn test_assert_contains() {
        let s = "hello world";
        assert_contains!(s, "world");
    }
}

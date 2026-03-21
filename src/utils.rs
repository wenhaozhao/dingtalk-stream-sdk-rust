//! Utility functions for DingTalk Stream SDK

/// Get the local IP address of the machine
pub fn get_local_ip() -> Option<String> {
    // Try to connect to a known IP to get the local address
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|addr| addr.ip().to_string())
}

/// Convert a URL with query parameters
pub fn build_url_with_params(base_url: &str, params: &[(&str, &str)]) -> String {
    let mut url = base_url.to_string();
    if !params.is_empty() {
        url.push('?');
        for (i, (key, value)) in params.iter().enumerate() {
            if i > 0 {
                url.push('&');
            }
            url.push_str(&format!("{}={}", key, urlencoding::encode(value)));
        }
    }
    url
}

/// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for byte in s.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    result.push(byte as char);
                }
                _ => {
                    result.push_str(&format!("%{:02X}", byte));
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding::encode("hello"), "hello");
        assert_eq!(urlencoding::encode("hello world"), "hello%20world");
        assert_eq!(urlencoding::encode("a+b"), "a%2Bb");
    }

    #[test]
    fn test_build_url_with_params() {
        let url = build_url_with_params("https://api.example.com", &[
            ("key", "value"),
            ("foo", "bar"),
        ]);
        assert!(url.contains("key=value"));
        assert!(url.contains("foo=bar"));
    }
}
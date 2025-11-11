use wget_faster_lib::{Cookie, CookieJar};
use std::path::PathBuf;

#[test]
fn test_cookie_creation() {
    let cookie = Cookie {
        domain: "example.com".to_string(),
        include_subdomains: true,
        path: "/".to_string(),
        secure: false,
        expiration: None,
        name: "session".to_string(),
        value: "abc123".to_string(),
    };

    assert_eq!(cookie.domain, "example.com");
    assert_eq!(cookie.name, "session");
    assert_eq!(cookie.value, "abc123");
    assert!(cookie.include_subdomains);
    assert!(!cookie.secure);
}

#[test]
fn test_cookie_jar_add_and_get() {
    let mut jar = CookieJar::new();

    jar.add_cookie(Cookie {
        domain: "example.com".to_string(),
        include_subdomains: true,
        path: "/".to_string(),
        secure: false,
        expiration: None,
        name: "session".to_string(),
        value: "abc123".to_string(),
    });

    let cookies = jar.get_cookies_for_domain("example.com");
    assert_eq!(cookies.len(), 1);
    assert_eq!(cookies[0].name, "session");
    assert_eq!(cookies[0].value, "abc123");
}

#[test]
fn test_cookie_jar_subdomain_matching() {
    let mut jar = CookieJar::new();

    jar.add_cookie(Cookie {
        domain: ".example.com".to_string(),
        include_subdomains: true,
        path: "/".to_string(),
        secure: false,
        expiration: None,
        name: "session".to_string(),
        value: "abc123".to_string(),
    });

    // Should match both example.com and subdomains
    assert_eq!(jar.get_cookies_for_domain("example.com").len(), 1);
    assert_eq!(jar.get_cookies_for_domain("www.example.com").len(), 1);
    assert_eq!(jar.get_cookies_for_domain("api.example.com").len(), 1);

    // Should not match different domain
    assert_eq!(jar.get_cookies_for_domain("other.com").len(), 0);
}

#[test]
fn test_cookie_jar_multiple_cookies() {
    let mut jar = CookieJar::new();

    jar.add_cookie(Cookie {
        domain: "example.com".to_string(),
        include_subdomains: false,
        path: "/".to_string(),
        secure: false,
        expiration: None,
        name: "cookie1".to_string(),
        value: "value1".to_string(),
    });

    jar.add_cookie(Cookie {
        domain: "example.com".to_string(),
        include_subdomains: false,
        path: "/api".to_string(),
        secure: true,
        expiration: None,
        name: "cookie2".to_string(),
        value: "value2".to_string(),
    });

    let cookies = jar.get_cookies_for_domain("example.com");
    assert_eq!(cookies.len(), 2);
}

#[test]
fn test_cookie_to_header() {
    let mut jar = CookieJar::new();

    jar.add_cookie(Cookie {
        domain: "example.com".to_string(),
        include_subdomains: false,
        path: "/".to_string(),
        secure: false,
        expiration: None,
        name: "cookie1".to_string(),
        value: "value1".to_string(),
    });

    jar.add_cookie(Cookie {
        domain: "example.com".to_string(),
        include_subdomains: false,
        path: "/".to_string(),
        secure: false,
        expiration: None,
        name: "cookie2".to_string(),
        value: "value2".to_string(),
    });

    let header = jar.to_cookie_header("example.com", "/", false);
    assert!(header.is_some());

    let header_value = header.unwrap();
    assert!(header_value.contains("cookie1=value1"));
    assert!(header_value.contains("cookie2=value2"));
    assert!(header_value.contains("; "));
}

#[test]
fn test_cookie_path_matching() {
    let mut jar = CookieJar::new();

    jar.add_cookie(Cookie {
        domain: "example.com".to_string(),
        include_subdomains: false,
        path: "/api".to_string(),
        secure: false,
        expiration: None,
        name: "api_cookie".to_string(),
        value: "api_value".to_string(),
    });

    // Should match /api and /api/...
    let header1 = jar.to_cookie_header("example.com", "/api", false);
    assert!(header1.is_some());

    let header2 = jar.to_cookie_header("example.com", "/api/users", false);
    assert!(header2.is_some());

    // Should not match different path
    let header3 = jar.to_cookie_header("example.com", "/other", false);
    assert!(header3.is_none());
}

#[test]
fn test_cookie_secure_flag() {
    let mut jar = CookieJar::new();

    jar.add_cookie(Cookie {
        domain: "example.com".to_string(),
        include_subdomains: false,
        path: "/".to_string(),
        secure: true,
        expiration: None,
        name: "secure_cookie".to_string(),
        value: "secure_value".to_string(),
    });

    // Should work with HTTPS (secure=true)
    let header1 = jar.to_cookie_header("example.com", "/", true);
    assert!(header1.is_some());

    // Should not work with HTTP (secure=false)
    let header2 = jar.to_cookie_header("example.com", "/", false);
    assert!(header2.is_none());
}

#[test]
fn test_parse_set_cookie() {
    let mut jar = CookieJar::new();

    jar.add_from_set_cookie(
        "example.com",
        "session=abc123; Path=/; Secure; Max-Age=3600",
    );

    let cookies = jar.get_cookies_for_domain("example.com");
    assert_eq!(cookies.len(), 1);
    assert_eq!(cookies[0].name, "session");
    assert_eq!(cookies[0].value, "abc123");
    assert_eq!(cookies[0].path, "/");
    assert!(cookies[0].secure);
    assert!(cookies[0].expiration.is_some());
}

#[test]
fn test_parse_set_cookie_with_domain() {
    let mut jar = CookieJar::new();

    jar.add_from_set_cookie(
        "example.com",
        "session=xyz789; Domain=.example.com; Path=/admin",
    );

    let cookies = jar.get_cookies_for_domain(".example.com");
    assert_eq!(cookies.len(), 1);
    assert_eq!(cookies[0].domain, ".example.com");
    assert_eq!(cookies[0].path, "/admin");
}

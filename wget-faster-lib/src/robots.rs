/// robots.txt parser for web crawling compliance
///
/// Implements robots.txt protocol (RFC 9309) for respectful web crawling.
/// Supports:
/// - User-agent specific rules
/// - Allow and Disallow directives
/// - Wildcard matching (*) in user-agent
/// - Most specific path matching

use std::collections::HashMap;

/// robots.txt parser
#[derive(Debug, Clone)]
pub struct RobotsTxt {
    /// Rules grouped by user-agent
    rules: HashMap<String, Vec<RobotRule>>,
}

/// A single robot rule (Allow or Disallow)
#[derive(Debug, Clone)]
struct RobotRule {
    /// Path pattern to match against
    path: String,
    /// Whether this is an Allow (true) or Disallow (false) rule
    allow: bool,
}

impl RobotsTxt {
    /// Parse robots.txt content from string
    ///
    /// # Arguments
    ///
    /// * `content` - The full content of robots.txt file
    ///
    /// # Returns
    ///
    /// Returns a parsed RobotsTxt instance
    ///
    /// # Examples
    ///
    /// ```
    /// use wget_faster_lib::robots::RobotsTxt;
    ///
    /// let content = r#"
    /// User-agent: *
    /// Disallow: /private/
    /// Allow: /public/
    /// "#;
    ///
    /// let robots = RobotsTxt::parse(content);
    /// assert!(!robots.is_allowed("/private/file.html", "MyBot"));
    /// assert!(robots.is_allowed("/public/file.html", "MyBot"));
    /// ```
    pub fn parse(content: &str) -> Self {
        let mut rules: HashMap<String, Vec<RobotRule>> = HashMap::new();
        let mut current_agents: Vec<String> = Vec::new();

        for line in content.lines() {
            // Remove comments and trim
            let line = if let Some(pos) = line.find('#') {
                &line[..pos]
            } else {
                line
            };
            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            // Parse line as "field: value"
            if let Some((field, value)) = line.split_once(':') {
                let field = field.trim().to_lowercase();
                let value = value.trim();

                match field.as_str() {
                    "user-agent" => {
                        // New user-agent section
                        let agent = value.to_lowercase();
                        if !current_agents.contains(&agent) {
                            current_agents.push(agent);
                        }
                    }
                    "disallow" => {
                        // Add Disallow rule to all current agents
                        for agent in &current_agents {
                            rules
                                .entry(agent.clone())
                                .or_default()
                                .push(RobotRule {
                                    path: value.to_string(),
                                    allow: false,
                                });
                        }
                    }
                    "allow" => {
                        // Add Allow rule to all current agents
                        for agent in &current_agents {
                            rules
                                .entry(agent.clone())
                                .or_default()
                                .push(RobotRule {
                                    path: value.to_string(),
                                    allow: true,
                                });
                        }
                    }
                    _ => {
                        // Ignore other fields (Crawl-delay, Sitemap, etc.)
                    }
                }
            }
        }

        Self { rules }
    }

    /// Check if a URL path is allowed for a given user-agent
    ///
    /// # Arguments
    ///
    /// * `path` - URL path to check (e.g., "/some/page.html")
    /// * `user_agent` - User-agent string to match against
    ///
    /// # Returns
    ///
    /// Returns `true` if the path is allowed, `false` if disallowed
    ///
    /// # Matching Rules
    ///
    /// 1. Tries to find rules for the specific user-agent first
    /// 2. Falls back to wildcard "*" rules if no specific match
    /// 3. Applies the most specific path match
    /// 4. Defaults to allow if no rules match
    ///
    /// # Examples
    ///
    /// ```
    /// use wget_faster_lib::robots::RobotsTxt;
    ///
    /// let robots = RobotsTxt::parse("User-agent: *\nDisallow: /admin/\n");
    /// assert!(!robots.is_allowed("/admin/panel", "wget"));
    /// assert!(robots.is_allowed("/public/page", "wget"));
    /// ```
    pub fn is_allowed(&self, path: &str, user_agent: &str) -> bool {
        // Try to find rules for specific user agent first, then fall back to "*"
        let agent_rules = self
            .rules
            .get(&user_agent.to_lowercase())
            .or_else(|| self.rules.get("*"));

        if let Some(rules) = agent_rules {
            // Process rules in order - most specific match wins
            // In robots.txt, more specific paths take precedence
            let mut best_match_len = 0;
            let mut best_match_allow = true; // Default to allow if no matches

            for rule in rules {
                if rule.path.is_empty() {
                    // Empty Disallow means allow everything
                    if !rule.allow {
                        continue;
                    }
                }

                // Check if path starts with the rule path
                if path.starts_with(&rule.path) {
                    let match_len = rule.path.len();
                    // Longer matches are more specific
                    if match_len > best_match_len {
                        best_match_len = match_len;
                        best_match_allow = rule.allow;
                    }
                }
            }

            // If we found any match, use it; otherwise default to allow
            if best_match_len > 0 {
                return best_match_allow;
            }
        }

        // Default: allow if no rules match
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let content = r#"
User-agent: *
Disallow: /private/
Allow: /public/
"#;
        let robots = RobotsTxt::parse(content);

        assert!(!robots.is_allowed("/private/file.html", "TestBot"));
        assert!(robots.is_allowed("/public/file.html", "TestBot"));
        assert!(robots.is_allowed("/other/file.html", "TestBot"));
    }

    #[test]
    fn test_specific_user_agent() {
        let content = r#"
User-agent: BadBot
Disallow: /

User-agent: GoodBot
Allow: /
"#;
        let robots = RobotsTxt::parse(content);

        assert!(!robots.is_allowed("/anything", "badbot"));
        assert!(robots.is_allowed("/anything", "goodbot"));
        assert!(robots.is_allowed("/anything", "otherbot")); // No rules for otherbot
    }

    #[test]
    fn test_empty_disallow() {
        let content = r#"
User-agent: *
Disallow:
"#;
        let robots = RobotsTxt::parse(content);

        // Empty Disallow means allow everything
        assert!(robots.is_allowed("/anything", "TestBot"));
    }

    #[test]
    fn test_most_specific_match() {
        let content = r#"
User-agent: *
Disallow: /admin/
Allow: /admin/public/
"#;
        let robots = RobotsTxt::parse(content);

        assert!(!robots.is_allowed("/admin/secret", "TestBot"));
        assert!(robots.is_allowed("/admin/public/page", "TestBot"));
    }

    #[test]
    fn test_comments_and_whitespace() {
        let content = r#"
# This is a comment
User-agent: *  # Inline comment
Disallow: /test/   # Another comment

"#;
        let robots = RobotsTxt::parse(content);

        assert!(!robots.is_allowed("/test/file", "TestBot"));
        assert!(robots.is_allowed("/other/file", "TestBot"));
    }

    #[test]
    fn test_no_rules() {
        let robots = RobotsTxt::parse("");
        assert!(robots.is_allowed("/anything", "TestBot"));
    }

    #[test]
    fn test_case_insensitive_user_agent() {
        let content = r#"
User-agent: MyBot
Disallow: /private/
"#;
        let robots = RobotsTxt::parse(content);

        assert!(!robots.is_allowed("/private/file", "mybot"));
        assert!(!robots.is_allowed("/private/file", "MYBOT"));
        assert!(!robots.is_allowed("/private/file", "MyBot"));
    }
}

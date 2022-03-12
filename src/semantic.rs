use git2::Commit;
use regex::Regex;
use semver::Version;

#[derive(Clone, Debug)]
pub struct Semantic {
    major: bool,
    minor: bool,
    patch: bool,
    semver: Version,
}

impl Default for Semantic {
    fn default() -> Self {
        Self {
            major: false,
            minor: false,
            patch: false,
            semver: Version::new(0, 0, 0),
        }
    }
}

impl Semantic {
    pub fn builder() -> Builder {
        Builder {
            semantic: Default::default(),
        }
    }

    fn new_major(&mut self) -> &mut Self {
        log::trace!("New major release.");
        self.major = true;
        self
    }

    fn new_minor(&mut self) -> &mut Self {
        log::trace!("New minor release.");
        self.minor = true;
        self
    }

    fn new_patch(&mut self) -> &mut Self {
        log::trace!("New patch release.");
        self.patch = true;
        self
    }

    pub fn is_major(&self) -> bool {
        self.major
    }

    pub fn is_minor(&self) -> bool {
        self.minor
    }

    pub fn is_patch(&self) -> bool {
        self.patch
    }
}

#[derive(Clone)]
pub struct Builder {
    semantic: Semantic,
}

impl Builder {
    fn semantic_major(&self, message: &str) -> bool {
        // todo: implement pattern matching for major releases -> static?
        log::trace!("Check header for major release: {:?}", message);
        message.contains("!:")
    }

    fn semantic_minor(&self, message: &str) -> bool {
        // todo: implement pattern matching for minor releases -> static?
        let re = Regex::new(r"^feat.*:").unwrap();
        log::trace!("Check header for minor release: {:?}", message);
        re.is_match(message)
    }

    fn semantic_patch(&self, message: &str) -> bool {
        // todo: implement pattern matching for patch releases -> static?
        let re = Regex::new(r"^fix.*:").unwrap();
        log::trace!("Check header for patch release: {:?}", message);
        re.is_match(message)
    }

    fn message_contains_semantic_information(&mut self, message: &str) -> &Self {
        if self.semantic_major(message) {
            // self.new_major();
            self.semantic.new_major();
        } else if self.semantic_minor(message) {
            // self.semantic =
            self.semantic.new_minor();
        } else if self.semantic_patch(message) {
            // self.semantic =
            self.semantic.new_patch();
        } else {
            log::trace!("No valuable semantic information from commit message.");
        }
        self
    }

    pub fn is_major(&self) -> bool {
        self.semantic.is_major()
    }

    pub fn analyze_commit(&mut self, commit: Commit<'_>) -> &Self {
        match commit.message_raw() {
            Some(message) => {
                self.message_contains_semantic_information(message);
            }
            None => {
                log::warn!("Commit message of '{:?}' is not valid.", commit.id());
            }
        }
        self
    }

    pub fn build(&self) -> Semantic {
        log::debug!("Build object: {:?}", self.semantic);
        self.semantic.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod given_message_contains {
        use super::*;
        mod semantic_major_information {
            use super::*;

            #[test]
            fn with_feat_break_then_semantic_major_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat!: sample commit message")
                    .build();
                assert!(semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }

            #[test]
            fn with_refactor_break_then_semantic_major_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("refactor!: sample commit message")
                    .build();
                assert!(semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }

            #[test]
            fn with_scope_and_break_then_semantic_major_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat(scope)!: sample commit message")
                    .build();
                assert!(semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }
        }

        mod semantic_minor_information {
            use super::*;

            #[test]
            fn with_feat_then_semantic_minor_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat: sample commit message")
                    .build();
                assert!(!semantic.is_major());
                assert!(semantic.is_minor());
                assert!(!semantic.is_patch());
            }

            #[test]
            fn with_feat_scope_then_semantic_minor_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat(scope): sample commit message")
                    .build();
                assert!(!semantic.is_major());
                assert!(semantic.is_minor());
                assert!(!semantic.is_patch());
            }
        }

        mod semantic_patch_information {
            use super::*;

            #[test]
            fn with_fix_then_semantic_patch_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("fix: sample commit message")
                    .build();
                assert!(!semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(semantic.is_patch());
            }

            #[test]
            fn with_fix_scope_then_semantic_patch_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("fix(scope): sample commit message")
                    .build();
                assert!(!semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(semantic.is_patch());
            }
        }

        mod irrelevant_semantic_information {
            use super::*;

            #[test]
            fn with_chore_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("chore: sample commit message")
                    .build();
                assert!(!semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }

            #[test]
            fn with_chore_scope_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("chore(scope): sample commit message")
                    .build();
                assert!(!semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }

            #[test]
            fn with_chore_scope_feat_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("chore(feat): sample commit message")
                    .build();
                assert!(!semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }
        }

        mod no_semantic_information {
            use super::*;

            #[test]
            fn with_non_convention_message_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feature sample commit message")
                    .build();
                assert!(!semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }

            #[test]
            fn with_empty_message_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("")
                    .build();
                assert!(!semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }

            #[test]
            fn with_merge_message_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("Merge feat: sample commit message")
                    .build();
                assert!(!semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }

            #[test]
            fn with_merge_message_and_scope_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information(
                        "Merge feat(scope): sample commit message",
                    )
                    .build();
                assert!(!semantic.is_major());
                assert!(!semantic.is_minor());
                assert!(!semantic.is_patch());
            }
        }
    }
}

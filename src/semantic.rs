use git2::{Commit, Reference};
use regex::Regex;
use semver::{BuildMetadata, Prerelease, Version};

use crate::error::SemVerError;

#[derive(Clone, Debug)]
pub struct Semantic {
    pub major: bool,
    pub minor: bool,
    pub patch: bool,
    pub prerelase: bool,
    pub version: Version,
}

impl Default for Semantic {
    fn default() -> Self {
        Self {
            major: false,
            minor: false,
            patch: false,
            prerelase: false,
            version: Version::new(0, 0, 0),
        }
    }
}

impl Semantic {
    pub fn builder() -> Builder {
        Builder {
            semantic: Default::default(),
        }
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

    fn message_contains_semantic_information(&mut self, message: &str) -> &mut Self {
        if self.semantic_major(message) {
            log::trace!("New major release.");
            self.semantic.major = true;
        } else if self.semantic_minor(message) {
            log::trace!("New minor release.");
            self.semantic.minor = true;
        } else if self.semantic_patch(message) {
            log::trace!("New patch release.");
            self.semantic.patch = true;
        } else {
            log::trace!("No valuable semantic information from commit message.");
        }
        self
    }

    pub fn is_prerelease(&mut self, branchname: &str) -> &mut Self {
        // todo: configuration branch - release stage mapping
        let main_release_branch = branchname.contains("main") || branchname.contains("master");
        if !main_release_branch || branchname.is_empty() {
            self.semantic.prerelase = true;
        }
        self
    }

    pub fn has_major_release(&self) -> bool {
        self.semantic.major
    }

    pub fn analyze_commit(&mut self, commit: Commit<'_>) -> &mut Self {
        match commit.message_raw() {
            Some(message) => {
                log::debug!("Check commit for semantic information: {:?}", message);
                self.message_contains_semantic_information(message);
            }
            None => {
                log::warn!("Commit message of '{:?}' is not valid.", commit.id());
            }
        }
        self
    }

    pub fn calculate_version(&mut self) -> &mut Self {
        let (mut major, mut minor, mut patch, mut prerelease) = (0, 0, 0, Prerelease::EMPTY);
        if self.semantic.major {
            major = self.semantic.version.major + 1;
        } else if self.semantic.minor {
            major = self.semantic.version.major;
            minor = self.semantic.version.minor + 1;
        } else if self.semantic.patch {
            major = self.semantic.version.major;
            minor = self.semantic.version.minor;
            patch = self.semantic.version.patch + 1;
        }
        if self.semantic.prerelase {
            // Todo: format of prerelease and how to calculate it
            let re = Regex::new(r"([A-Za-z\-\.]+)(?:([\d]*))").unwrap();
            prerelease = match re.captures(self.semantic.version.pre.as_str()) {
                Some(caps) => {
                    let text = caps.get(1).map_or("", |m| m.as_str());
                    let mut number = caps
                        .get(2)
                        .map_or("", |m| m.as_str())
                        .parse::<i32>()
                        .unwrap_or(0);
                    number += 1;
                    Prerelease::new(&format!("{}{}", text, number)).unwrap()
                }
                None => {
                    log::warn!(
                        "Could not parse prerelease from: {:?}",
                        self.semantic.prerelase
                    );
                    Prerelease::new("pre.0").unwrap()
                }
            };
        }
        self.semantic.version = Version {
            major,
            minor,
            patch,
            pre: prerelease,
            build: BuildMetadata::EMPTY,
        };
        self
    }

    pub fn previous_version(&mut self, version: &str) -> Result<&mut Self, SemVerError> {
        let version = Version::parse(version)?;
        self.semantic.version = version;
        Ok(self)
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
                assert!(semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }

            #[test]
            fn with_refactor_break_then_semantic_major_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("refactor!: sample commit message")
                    .build();
                assert!(semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }

            #[test]
            fn with_scope_and_break_then_semantic_major_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat(scope)!: sample commit message")
                    .build();
                assert!(semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }
        }

        mod semantic_minor_information {
            use super::*;

            #[test]
            fn with_feat_then_semantic_minor_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat: sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(semantic.minor);
                assert!(!semantic.patch);
            }

            #[test]
            fn with_feat_scope_then_semantic_minor_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat(scope): sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(semantic.minor);
                assert!(!semantic.patch);
            }
        }

        mod semantic_patch_information {
            use super::*;

            #[test]
            fn with_fix_then_semantic_patch_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("fix: sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(!semantic.minor);
                assert!(semantic.patch);
            }

            #[test]
            fn with_fix_scope_then_semantic_patch_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("fix(scope): sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(!semantic.minor);
                assert!(semantic.patch);
            }
        }

        mod irrelevant_semantic_information {
            use super::*;

            #[test]
            fn with_chore_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("chore: sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }

            #[test]
            fn with_chore_scope_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("chore(scope): sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }

            #[test]
            fn with_chore_scope_feat_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("chore(feat): sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }
        }

        mod no_semantic_information {
            use super::*;

            #[test]
            fn with_non_convention_message_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feature sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }

            #[test]
            fn with_empty_message_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("")
                    .build();
                assert!(!semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }

            #[test]
            fn with_merge_message_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("Merge feat: sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }

            #[test]
            fn with_merge_message_and_scope_then_no_semantic_information_is_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information(
                        "Merge feat(scope): sample commit message",
                    )
                    .build();
                assert!(!semantic.major);
                assert!(!semantic.minor);
                assert!(!semantic.patch);
            }
        }

        mod multiple_semantic_statements {
            use super::*;

            #[test]
            fn with_feat_and_fix_then_both_are_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat: sample commit message")
                    .message_contains_semantic_information("fix: sample commit message")
                    .build();
                assert!(!semantic.major);
                assert!(semantic.minor);
                assert!(semantic.patch);
            }

            #[test]
            fn with_feat_break_feat_and_fix_then_all_are_set() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat!: sample breaking commit message")
                    .message_contains_semantic_information("feat: sample commit message")
                    .message_contains_semantic_information("fix: sample commit message")
                    .build();
                assert!(semantic.major);
                assert!(semantic.minor);
                assert!(semantic.patch);
            }
        }
    }

    mod given_message_indicates {
        use super::*;
        mod semantic_major_is_calculated {
            use super::*;

            #[test]
            fn then_major_is_increased() {
                let semantic = Semantic::builder()
                    .previous_version("1.0.0")
                    .unwrap()
                    .message_contains_semantic_information("feat!: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(2, semantic.version.major);
            }

            #[test]
            fn then_minor_patch_is_reset() {
                let semantic = Semantic::builder()
                    .previous_version("1.2.3")
                    .unwrap()
                    .message_contains_semantic_information("feat!: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(2, semantic.version.major);
                assert_eq!(0, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
            }

            #[test]
            fn with_no_version_then_major_is_increased() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat!: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(1, semantic.version.major);
                assert_eq!(0, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
            }
        }

        mod semantic_minor_is_calculated {
            use super::*;

            #[test]
            fn then_minor_is_increased() {
                let semantic = Semantic::builder()
                    .previous_version("1.2.0")
                    .unwrap()
                    .message_contains_semantic_information("feat: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(3, semantic.version.minor);
            }

            #[test]
            fn then_patch_is_reset() {
                let semantic = Semantic::builder()
                    .previous_version("1.2.0")
                    .unwrap()
                    .message_contains_semantic_information("feat: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(3, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
            }

            #[test]
            fn then_major_is_unchanged() {
                let semantic = Semantic::builder()
                    .previous_version("1.2.0")
                    .unwrap()
                    .message_contains_semantic_information("feat: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(1, semantic.version.major);
                assert_eq!(3, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
            }

            #[test]
            fn with_no_version_then_minor_is_increased() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(0, semantic.version.major);
                assert_eq!(1, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
            }
        }

        mod semantic_patch_is_calculated {
            use super::*;

            #[test]
            fn then_patch_is_increased() {
                let semantic = Semantic::builder()
                    .previous_version("1.2.5")
                    .unwrap()
                    .message_contains_semantic_information("fix: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(6, semantic.version.patch);
            }

            #[test]
            fn then_major_minor_is_unchanged() {
                let semantic = Semantic::builder()
                    .previous_version("1.2.5")
                    .unwrap()
                    .message_contains_semantic_information("fix: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(1, semantic.version.major);
                assert_eq!(2, semantic.version.minor);
                assert_eq!(6, semantic.version.patch);
            }

            #[test]
            fn with_no_version_then_patch_is_increased() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("fix: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(0, semantic.version.major);
                assert_eq!(0, semantic.version.minor);
                assert_eq!(1, semantic.version.patch);
            }
        }

        mod multiple_semantic_statements {
            use super::*;

            #[test]
            fn with_feat_and_fix_then_minor_is_increased_and_patch_reset() {
                let semantic = Semantic::builder()
                    .previous_version("1.2.5")
                    .unwrap()
                    .message_contains_semantic_information("feat: sample commit message")
                    .message_contains_semantic_information("fix: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(1, semantic.version.major);
                assert_eq!(3, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
            }

            #[test]
            fn with_feat_break_feat_and_fix_then_major_is_increased_others_reset() {
                let semantic = Semantic::builder()
                    .previous_version("1.2.5")
                    .unwrap()
                    .message_contains_semantic_information("feat!: sample breaking commit message")
                    .message_contains_semantic_information("feat: sample commit message")
                    .message_contains_semantic_information("fix: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(2, semantic.version.major);
                assert_eq!(0, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
            }

            #[test]
            fn with_no_version_and_feat_and_fix_then_minor_is_increased() {
                let semantic = Semantic::builder()
                    .message_contains_semantic_information("feat: sample commit message")
                    .message_contains_semantic_information("fix: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(0, semantic.version.major);
                assert_eq!(1, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
            }
        }
    }

    mod given_branchname {
        use super::*;

        #[test]
        fn is_main_then_prerelease_is_not_set() {
            let semantic = Semantic::builder().is_prerelease("refs/heads/main").build();
            assert!(!semantic.prerelase);
        }

        #[test]
        fn is_master_then_prerelease_is_not_set() {
            let semantic = Semantic::builder().is_prerelease("master").build();
            assert!(!semantic.prerelase);
        }

        #[test]
        fn is_develop_then_prerelease_is_set() {
            let semantic = Semantic::builder().is_prerelease("develop").build();
            assert!(semantic.prerelase);
        }

        #[test]
        fn is_undefined_then_prerelease_is_set() {
            let semantic = Semantic::builder().is_prerelease("").build();
            assert!(semantic.prerelase);
        }
    }

    mod given_prerelease_and_message_indicates {
        use super::*;
        mod when_no_prerelease_was_present {
            use super::*;

            #[test]
            fn then_prerelease_is_initialized() {
                let semantic = Semantic::builder()
                    .previous_version("1.0.0")
                    .unwrap()
                    .is_prerelease("develop")
                    .message_contains_semantic_information("feat!: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(2, semantic.version.major);
                assert_eq!(0, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
                assert_eq!("pre.0", semantic.version.pre.as_str());
            }
        }

        mod when_invalid_prerelease_was_present {
            use super::*;

            #[test]
            fn then_prerelase_is_normalized_with_buildnumber() {
                let semantic = Semantic::builder()
                    .previous_version("1.0.0-pre")
                    .unwrap()
                    .is_prerelease("develop")
                    .message_contains_semantic_information("feat!: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(2, semantic.version.major);
                assert_eq!(0, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
                assert_eq!("pre1", semantic.version.pre.as_str());
            }

            #[test]
            fn then_some_prerelease_tag_is_normalized_with_buildnumber() {
                let semantic = Semantic::builder()
                    .previous_version("1.0.0-shitshow")
                    .unwrap()
                    .is_prerelease("develop")
                    .message_contains_semantic_information("feat!: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(2, semantic.version.major);
                assert_eq!(0, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
                assert_eq!("shitshow1", semantic.version.pre.as_str());
            }
        }

        mod when_prerelease_was_present {
            use super::*;

            #[test]
            fn then_prerelase_with_dot_is_parsed() {
                let semantic = Semantic::builder()
                    .previous_version("1.0.0-pre.2")
                    .unwrap()
                    .is_prerelease("develop")
                    .message_contains_semantic_information("feat!: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(2, semantic.version.major);
                assert_eq!(0, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
                assert_eq!("pre.3", semantic.version.pre.as_str());
            }

            #[test]
            fn then_alpha_with_dot_is_parsed() {
                let semantic = Semantic::builder()
                    .previous_version("1.0.0-alpha.2")
                    .unwrap()
                    .is_prerelease("develop")
                    .message_contains_semantic_information("feat: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(1, semantic.version.major);
                assert_eq!(1, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
                assert_eq!("alpha.3", semantic.version.pre.as_str());
            }

            #[test]
            fn then_alpha_without_dot_is_parsed() {
                let semantic = Semantic::builder()
                    .previous_version("1.0.0-alpha2")
                    .unwrap()
                    .is_prerelease("develop")
                    .message_contains_semantic_information("feat: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(1, semantic.version.major);
                assert_eq!(1, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
                assert_eq!("alpha3", semantic.version.pre.as_str());
            }

            #[test]
            fn then_minus_seperated_prerelease_without_dot_is_parsed() {
                let semantic = Semantic::builder()
                    .previous_version("1.0.0-alpha-or-whatever2")
                    .unwrap()
                    .is_prerelease("develop")
                    .message_contains_semantic_information("feat: sample commit message")
                    .calculate_version()
                    .build();
                assert_eq!(1, semantic.version.major);
                assert_eq!(1, semantic.version.minor);
                assert_eq!(0, semantic.version.patch);
                assert_eq!("alpha-or-whatever3", semantic.version.pre.as_str());
            }
        }
    }
}

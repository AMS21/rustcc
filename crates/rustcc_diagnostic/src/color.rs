// Conditional color support: when the `color` feature is enabled, re-export
// colored::Colorize. Otherwise, provide a no-op shim with the same trait name
// so call sites can continue to import `rustcc_diagnostic::color::Colorize`.

#[cfg(feature = "color")]
pub use colored::Colorize;

#[cfg(not(feature = "color"))]
mod shim {
    pub trait Colorize: Sized {
        #[must_use]
        fn red(self) -> Self {
            self
        }

        #[must_use]
        fn yellow(self) -> Self {
            self
        }

        #[must_use]
        fn bold(self) -> Self {
            self
        }

        #[must_use]
        fn italic(self) -> Self {
            self
        }

        #[must_use]
        fn underline(self) -> Self {
            self
        }
    }

    impl Colorize for &str {}

    impl Colorize for String {}
}

#[cfg(not(feature = "color"))]
pub use shim::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "color"))]
    #[test]
    fn no_color_feature_is_noop_for_str() {
        let s = "hello";
        let out = s.red().bold().italic().underline().yellow();
        assert_eq!(out, s);
        // Ensure chaining on temporaries compiles and returns same content
        let chained = "x".bold().red().yellow();
        assert_eq!(chained, "x");
    }

    #[cfg(not(feature = "color"))]
    #[test]
    fn no_color_feature_is_noop_for_string() {
        let s = String::from("world");
        let out = s.clone().bold();
        assert_eq!(out, s);
        let out2 = out.red();
        assert_eq!(out2, "world");
    }

    #[cfg(feature = "color")]
    #[test]
    fn color_feature_formats_string() {
        // Force-enable colors to get stable output in test environments.
        // This API is provided by the `colored` crate.
        colored::control::set_override(true);

        let s = "hello";
        let out = s.bold().red().to_string();
        // The original content should be present
        assert!(out.contains(s));
        // And styling should change the string (e.g., add ANSI escapes)
        assert_ne!(out, s);
    }
}

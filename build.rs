// Compile the Obj-C notification helper on macOS. No-op on other platforms.

fn main() {
    #[cfg(target_os = "macos")]
    {
        let out_dir = std::env::var("OUT_DIR").unwrap();
        let output = format!("{}/ding-notifier", out_dir);

        let status = std::process::Command::new("clang")
            .args([
                "-fobjc-arc",
                "-O2",
                "-mmacosx-version-min=11.0",
                "-o",
                &output,
                "objc/notifier.m",
                "-framework",
                "UserNotifications",
                "-framework",
                "Foundation",
            ])
            .status()
            .expect("failed to run clang — is Xcode or CommandLineTools installed?");

        assert!(status.success(), "clang failed to compile objc/notifier.m");

        println!("cargo:rerun-if-changed=objc/notifier.m");
    }
}

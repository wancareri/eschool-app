//! Generates typed `res::` constants from `resource/` (https://daybrite.dev/docs/resources).
fn main() {
    day_build::generate_resources().expect("day-build: resource codegen");

    // Compile NSLog bridge for iOS so idevicesyslog can see our logs.
    #[cfg(target_os = "ios")]
    {
        cc::Build::new()
            .file("platform/ios/nslog_bridge.m")
            .compile("nslog_bridge");
    }
}

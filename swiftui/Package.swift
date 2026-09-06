// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "EschoolViews",
    platforms: [
        .iOS(.v16),
        .macOS(.v13)
    ],
    products: [
        .library(name: "EschoolViews", targets: ["EschoolViews"])
    ],
    targets: [
        .target(name: "EschoolViews")
    ]
)

// swift-tools-version: 5.9

import PackageDescription

let package = Package(
    name: "ArcadiaDevelopmentLauncher",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(
            name: "ArcadiaDevelopmentLauncher",
            targets: ["ArcadiaDevelopmentLauncher"]
        )
    ],
    targets: [
        .executableTarget(
            name: "ArcadiaDevelopmentLauncher"
        )
    ]
)

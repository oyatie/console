// swift-tools-version:6.1

import PackageDescription

let package = Package(
    name: "ConsoleAPIClient",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
        .tvOS(.v13),
        .watchOS(.v6),
        .visionOS(.v1),
    ],
    products: [
        .library(name: "ConsoleAPIClient", targets: ["ConsoleAPIClient"]),
    ],
    dependencies: [
        .package(url: "https://github.com/apple/swift-openapi-runtime", exact: "1.12.0"),
    ],
    targets: [
        .target(
            name: "ConsoleAPIClient",
            dependencies: [
                .product(name: "OpenAPIRuntime", package: "swift-openapi-runtime"),
            ]
        ),
        .executableTarget(
            name: "ConsoleAPIClientContractTests",
            dependencies: ["ConsoleAPIClient"],
            path: "Tests/ConsoleAPIClientContractTests"
        ),
    ]
)

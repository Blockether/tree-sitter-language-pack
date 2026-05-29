// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "E2eSwift",
    platforms: [
        .macOS(.v13),
        .iOS(.v16),
    ],
    dependencies: [
        .binaryTarget(name: "TreeSitterLanguagePack", url: "https://github.com/kreuzberg-dev/tree-sitter-language-pack/releases/download/v1.9.0-rc.17/TreeSitterLanguagePack-rs.artifactbundle.zip", checksum: "__ALEF_SWIFT_CHECKSUM__"),
    ],
    targets: [
        .testTarget(
            name: "TreeSitterLanguagePackE2ETests",
            dependencies: [.binaryTarget(name: "TreeSitterLanguagePack")]
        ),
    ]
)

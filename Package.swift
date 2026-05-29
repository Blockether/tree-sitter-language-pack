// swift-tools-version: 6.0
// Root-level Package.swift — alef-generated for published distributions.
//
// This manifest uses `.binaryTarget` for pre-built XCFramework/artifact bundles.
// External consumers depend on this via `.package(url: "...", from: "...")`.
//
// For in-tree development, see `packages/swift/Package.swift` and
// `packages/swift/README.md` for the source-based workflow.
import PackageDescription

let package = Package(
  name: "TreeSitterLanguagePack",
  platforms: [
    .macOS(.v13),
    .iOS(.v16),
  ],
  products: [
    .library(name: "TreeSitterLanguagePack", targets: ["TreeSitterLanguagePack"])
  ],
  targets: [
    // RustBridge: pre-built binary target containing the compiled Rust library
    // for macOS (arm64, x86_64), iOS (device, simulator), and Linux (arm64, x86_64).
    // The binary includes C headers for swift-bridge interop.
    .binaryTarget(
      name: "RustBridge",
      url:
        "https://github.com/kreuzberg-dev/tree-sitter-language-pack/releases/download/v1.9.0-rc.16/TreeSitterLanguagePack-rs.artifactbundle.zip",
      checksum: "41b7ab8582e44edc13594435ce3483336731593bec61a24b917c540d6ea6208c"
    ),
    .target(
      name: "TreeSitterLanguagePack",
      dependencies: ["RustBridge"],
      path: "packages/swift/Sources/TreeSitterLanguagePack"
    ),
  ]
)

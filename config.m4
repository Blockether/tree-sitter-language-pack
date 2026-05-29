dnl Configuration for Rust-based PHP extension via ext-php-rs.
dnl Allows phpize to recognize this extension during source compilation (PIE fallback).

PHP_ARG_ENABLE([tree_sitter_language_pack],
  [whether to enable the tree_sitter_language_pack extension],
  [AS_HELP_STRING([--enable-tree_sitter_language_pack],
    [Enable tree_sitter_language_pack extension support])],
  [yes])

if test "$PHP_TREE_SITTER_LANGUAGE_PACK_ENABLED" = "yes"; then
  dnl Recognize the extension directory for phpize/make
  PHP_NEW_EXTENSION(tree_sitter_language_pack, [], $ext_shared)

  dnl Invoke cargo build to compile the Rust FFI library
  AC_CONFIG_COMMANDS([cargo-build], [
    if test -f "crates/tree-sitter-language-pack-php/Cargo.toml"; then
      cargo build --release --manifest-path crates/tree-sitter-language-pack-php/Cargo.toml || exit 1
      cargo_output_dir="crates/tree-sitter-language-pack-php/target/release"
      ext_soname="tree_sitter_language_pack"

      dnl Detect output filename based on platform
      if test -f "${cargo_output_dir}/libtree-sitter-language-pack_php.dylib"; then
        cargo_lib="${cargo_output_dir}/libtree-sitter-language-pack_php.dylib"
      elif test -f "${cargo_output_dir}/libtree-sitter-language-pack_php.so"; then
        cargo_lib="${cargo_output_dir}/libtree-sitter-language-pack_php.so"
      else
        AC_MSG_ERROR([cargo build succeeded but .so/.dylib not found])
      fi

      dnl Copy the compiled library to modules/ directory for phpize to install
      cp "${cargo_lib}" "modules/${ext_soname}.so" || exit 1
    else
      AC_MSG_ERROR([crates/tree-sitter-language-pack-php/Cargo.toml not found])
    fi
  ], [
    extension_name=tree_sitter_language_pack
  ])
fi

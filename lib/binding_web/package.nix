{
  wasm-test-grammars,
  lib,
  bun,
  stdenv,
  rustPlatform,
  cargo,
  pkg-config,
  emscripten,
  src,
  version,
}:
stdenv.mkDerivation {
  inherit src version;

  pname = "web-tree-sitter";

  nativeBuildInputs = [
    bun
    rustPlatform.cargoSetupHook
    cargo
    pkg-config
    emscripten
  ];

  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ../../Cargo.lock;
  };

  doCheck = true;

  buildPhase = ''
    pushd lib/binding_web

    bun install
    bun run build
    bun run build:debug

    popd

    mkdir -p target/release

    for grammar in ${wasm-test-grammars}/*.wasm; do
      if [ -f "$grammar" ]; then
        cp "$grammar" target/release/
      fi
    done
  '';

  checkPhase = ''
    cd lib/binding_web && bun test
  '';

  installPhase = ''
    mkdir -p $out
    cp -r lib/binding_web/web-tree-sitter.* $out/
    cp -r lib/binding_web/debug $out/ || true
  '';

  meta = {
    description = "web-tree-sitter - WebAssembly bindings to the Tree-sitter parsing library.";
    longDescription = ''
      web-tree-sitter provides WebAssembly bindings to the Tree-sitter parsing library.
      It can build a concrete syntax tree for a source file and efficiently update
      the syntax tree as the source file is edited. This package provides the WebAssembly bindings
      and a JavaScript API for using them in web browsers
    '';
    homepage = "https://tree-sitter.github.io/tree-sitter";
    changelog = "https://github.com/tree-sitter/tree-sitter/releases/tag/v${version}";
    license = lib.licenses.mit;
    maintainers = with lib.maintainers; [ amaanq ];
    platforms = lib.platforms.all;
  };
}

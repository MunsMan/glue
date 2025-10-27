{
  pkgs,
  cargoToml,
}:
{
  # Individual check for just Rust tests
  rust-tests = pkgs.rustPlatform.buildRustPackage {
    inherit (cargoToml.package) version name;
    src = ./.;
    cargoLock.lockFile = ./Cargo.lock;
    # Override the build phase to run tests instead
    buildPhase = ''
      cargo test --release
    '';

    # Create a dummy output
    installPhase = ''
      mkdir -p $out
      echo "Tests passed" > $out/test-results
    '';
  };

  # Individual check for just script tests
  script-tests = pkgs.stdenv.mkDerivation {
    name = "glue-script-tests";
    src = ./.;
    buildInputs = with pkgs; [
      python3
      wezterm
    ];
    buildPhase = ''
      mkdir -p $out/tests
      cp eww/scripts/wifi.sh $out/tests/
      cp eww/scripts/test_wifi.py $out/tests/
      cd $out/tests
      chmod +x wifi.sh test_wifi.py
    '';
    checkPhase = ''
      cd $out/tests
      echo "Running WiFi script tests..."
      python3 test_wifi.py
    '';
    doCheck = true;
    installPhase = ''
      echo "Script tests completed successfully" > $out/test-results
    '';
  };
}

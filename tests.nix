{ pkgs, rust }:
{
  # Combined test suite running both Rust and script tests
  all-tests = pkgs.stdenv.mkDerivation {
    name = "glue-all-tests";
    src = ./.;
    nativeBuildInputs = with pkgs; [
      rust
      pkg-config
      dbus
    ];
    buildInputs = with pkgs; [
      python3
      eww
      wezterm
    ];
    buildPhase = ''
      echo "Setting up test environment..."
      export CARGO_HOME=$(mktemp -d)
      
      # Copy source for tests
      mkdir -p $out/tests
      cp -r . $out/tests/
      cd $out/tests
      
      # Make scripts executable
      chmod +x eww/scripts/wifi.sh eww/scripts/test_wifi.py
    '';
    checkPhase = ''
      cd $out/tests
      
      echo "🦀 Running Rust tests..."
      echo "=========================="
      cargo test --verbose
      
      echo ""
      echo "📜 Running WiFi script tests..."
      echo "================================="
      cd eww/scripts
      python3 test_wifi.py
      
      echo ""
      echo "✅ All tests completed successfully!"
    '';
    doCheck = true;
    installPhase = ''
      echo "All tests passed at $(date)" > $out/test-results
      echo "Rust tests: PASSED" >> $out/test-results
      echo "Script tests: PASSED" >> $out/test-results
    '';
  };
  
  # Individual check for just Rust tests
  rust-tests = pkgs.stdenv.mkDerivation {
    name = "glue-rust-tests";
    src = ./.;
    nativeBuildInputs = with pkgs; [
      rust
      pkg-config
      dbus
    ];
    buildInputs = with pkgs; [
      eww
      wezterm
    ];
    buildPhase = ''
      export CARGO_HOME=$(mktemp -d)
      cp -r . $out/
      cd $out
    '';
    checkPhase = ''
      cd $out
      echo "Running Rust tests..."
      cargo test --verbose
    '';
    doCheck = true;
    installPhase = ''
      echo "Rust tests completed successfully" > $out/test-results
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
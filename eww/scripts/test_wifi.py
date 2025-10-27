#!/usr/bin/env python3
"""
Enhanced tests for wifi.sh script covering all functionality
"""

import subprocess
import unittest
from pathlib import Path


class TestWifiScript(unittest.TestCase):
    """Comprehensive test cases for wifi.sh script"""
    
    def setUp(self):
        """Set up test environment"""
        self.script_path = Path(__file__).parent / "wifi.sh"
        self.assertTrue(self.script_path.exists(), "wifi.sh script must exist")
    
    def run_script(self, *args):
        """Run the wifi script with given arguments"""
        cmd = [str(self.script_path)] + list(args)
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result.returncode, result.stdout.strip(), result.stderr.strip()
    
    def test_script_executable(self):
        """Test that script is executable"""
        self.assertTrue(self.script_path.stat().st_mode & 0o111, "Script should be executable")
    
    def test_tool_detection(self):
        """Test WiFi tool detection returns valid values"""
        returncode, output, _ = self.run_script("tool")
        self.assertEqual(returncode, 0, "Tool detection should succeed")
        self.assertIn(output, ["nm", "iwctl", "none"], f"Should return valid tool name, got: {output}")
    
    def test_basic_commands(self):
        """Test all basic commands return output without error"""
        basic_commands = ["tool", "color", "icon", "text", "connected"]
        
        for cmd in basic_commands:
            with self.subTest(command=cmd):
                returncode, output, stderr = self.run_script(cmd)
                self.assertEqual(returncode, 0, f"Command '{cmd}' should succeed")
                self.assertIsNotNone(output, f"Command '{cmd}' should return output")
    
    def test_connected_returns_boolean(self):
        """Test that 'connected' command returns true/false"""
        returncode, output, _ = self.run_script("connected")
        self.assertEqual(returncode, 0, "Connected command should succeed")
        self.assertIn(output, ["true", "false"], f"Connected should return boolean, got: {output}")
    
    def test_color_returns_hex(self):
        """Test that color command returns a hex color"""
        returncode, output, _ = self.run_script("color")
        self.assertEqual(returncode, 0, "Color command should succeed")
        self.assertTrue(output.startswith("#"), f"Color should start with #, got: {output}")
        self.assertEqual(len(output), 7, f"Color should be 7 characters (#rrggbb), got: {output}")
    
    def test_icon_returns_unicode(self):
        """Test that icon command returns a unicode icon"""
        returncode, output, _ = self.run_script("icon")
        self.assertEqual(returncode, 0, "Icon command should succeed")
        self.assertGreater(len(output), 0, "Icon should not be empty")
        # Check if it's a unicode character (WiFi icons are multi-byte)
        self.assertTrue(any(ord(char) > 127 for char in output), "Icon should contain unicode characters")
    
    def test_status_command(self):
        """Test comprehensive status command"""
        returncode, output, _ = self.run_script("status")
        self.assertEqual(returncode, 0, "Status command should succeed")
        self.assertIn("Tool:", output, "Status should include tool information")
        self.assertIn("Connected:", output, "Status should include connection status")
        self.assertIn("SSID:", output, "Status should include SSID information")
    
    def test_wezterm_integration(self):
        """Test that script properly integrates with wezterm"""
        script_content = self.script_path.read_text()
        
        # Check for wezterm usage
        self.assertIn("wezterm", script_content, "Script should contain 'wezterm'")
        self.assertIn("wezterm start", script_content, "Script should use 'wezterm start' command")
        self.assertIn("--class", script_content, "Script should use window class for wezterm")
        self.assertIn("wifi-tui", script_content, "Script should use wifi-tui class")
    
    def test_tui_tools_support(self):
        """Test that script supports both nmtui and impala/iwctl"""
        script_content = self.script_path.read_text()
        
        # Check for TUI tool support
        self.assertIn("nmtui", script_content, "Script should support nmtui")
        self.assertIn("impala", script_content, "Script should support impala")
        self.assertIn("iwctl", script_content, "Script should support iwctl")
    
    def test_signal_strength_icons(self):
        """Test that script contains different signal strength icons"""
        script_content = self.script_path.read_text()
        
        # Check for different WiFi icons (signal strength indicators)
        wifi_icons = ["󰤨", "󰤥", "󰤢", "󰤟", "󰖪"]
        for icon in wifi_icons:
            self.assertIn(icon, script_content, f"Script should contain WiFi icon: {icon}")
    
    def test_error_handling(self):
        """Test error handling with invalid arguments"""
        returncode, output, stderr = self.run_script("invalid_argument")
        self.assertNotEqual(returncode, 0, "Invalid argument should return non-zero exit code")
        self.assertIn("Usage:", output, "Should show usage information on invalid argument")
    
    def test_notification_support(self):
        """Test that script includes notification support"""
        script_content = self.script_path.read_text()
        self.assertIn("notify-send", script_content, "Script should use notify-send for notifications")
    
    def test_ansi_code_handling(self):
        """Test that script handles ANSI color codes properly"""
        script_content = self.script_path.read_text()
        # Check for ANSI code stripping pattern
        self.assertIn("sed 's/\\x1b\\[[0-9;]*m//g'", script_content, 
                     "Script should strip ANSI color codes from iwctl output")
    
    def test_help_command(self):
        """Test that script shows usage help"""
        # Test with no arguments
        returncode, output, _ = self.run_script()
        self.assertNotEqual(returncode, 0, "No arguments should return error")
        self.assertIn("Usage:", output, "Should show usage information")
        
        # Test available commands are listed
        expected_commands = ["color", "text", "icon", "connected", "tool", "toggle", "open", "tui", "status"]
        for cmd in expected_commands:
            self.assertIn(cmd, output, f"Usage should mention command: {cmd}")


if __name__ == "__main__":
    # Create a test suite with custom formatting
    loader = unittest.TestLoader()
    suite = loader.loadTestsFromTestCase(TestWifiScript)
    
    # Custom test runner with better formatting
    class ColoredTextTestResult(unittest.TextTestResult):
        def getDescription(self, test):
            doc_first_line = test.shortDescription()
            if doc_first_line:
                return f"{test._testMethodName}: {doc_first_line}"
            return test._testMethodName
        
        def startTest(self, test):
            super().startTest(test)
            if self.stream.isatty():
                self.stream.write(f"• {self.getDescription(test)} ... ")
                self.stream.flush()
        
        def addSuccess(self, test):
            super().addSuccess(test)
            if self.stream.isatty():
                self.stream.writeln("✓ PASS")
        
        def addError(self, test, err):
            super().addError(test, err)
            if self.stream.isatty():
                self.stream.writeln("✗ ERROR")
        
        def addFailure(self, test, err):
            super().addFailure(test, err)
            if self.stream.isatty():
                self.stream.writeln("✗ FAIL")
    
    class ColoredTextTestRunner(unittest.TextTestRunner):
        resultclass = ColoredTextTestResult
        
        def run(self, test):
            print("Running WiFi Script Tests")
            print("=" * 50)
            result = super().run(test)
            print("=" * 50)
            if result.wasSuccessful():
                print(f"✓ All {result.testsRun} tests passed!")
            else:
                failed = len(result.failures) + len(result.errors)
                print(f"✗ {failed}/{result.testsRun} tests failed")
            return result
    
    # Run the tests
    runner = ColoredTextTestRunner(verbosity=0)
    runner.run(suite)
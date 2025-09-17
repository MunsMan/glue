#!/usr/bin/env python3
"""
Simple Python tests for wifi.sh script - just 3 basic tests
"""

import subprocess
import unittest
from pathlib import Path


class TestWifiScript(unittest.TestCase):
    """Basic test cases for wifi.sh script"""
    
    def setUp(self):
        """Set up test environment"""
        self.script_path = Path(__file__).parent / "wifi.sh"
    
    def run_script(self, *args):
        """Run the wifi script with given arguments"""
        cmd = [str(self.script_path)] + list(args)
        result = subprocess.run(cmd, capture_output=True, text=True)
        return result.returncode, result.stdout.strip()
    
    def test_script_runs_without_error(self):
        """Test 1: Script executes without crashing"""
        returncode, output = self.run_script("tool")
        self.assertEqual(returncode, 0, "Script should run without error")
        self.assertIn(output, ["nm", "iwctl", "none"], "Should return valid tool name")
    
    def test_all_arguments_work(self):
        """Test 2: All main arguments return some output"""
        args_to_test = ["tool", "color", "icon", "text"]
        
        for arg in args_to_test:
            with self.subTest(arg=arg):
                returncode, output = self.run_script(arg)
                self.assertEqual(returncode, 0, f"Argument '{arg}' should work")
                # Don't check output content, just that it doesn't crash
    
    def test_wezterm_in_script(self):
        """Test 3: Script contains wezterm (our main requirement)"""
        script_content = self.script_path.read_text()
        self.assertIn("wezterm", script_content, "Script should contain 'wezterm'")
        self.assertIn("wezterm start", script_content, "Script should use 'wezterm start' command")


if __name__ == "__main__":
    unittest.main(verbosity=2)
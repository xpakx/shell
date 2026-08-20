from base import TestShellSuite
from pathlib import Path


class NavigationTests(TestShellSuite):
    def test_type_for_executables(self):
        """Verify pwd works"""
        self.shell.sendline("type pwd")
        result = self.expect_exact("pwd is a shell builtin")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("pwd")
        cwd = Path.cwd()
        result = self.expect_exact(cwd)
        self.verify_result(result, "Received expected message")

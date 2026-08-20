from base import TestShellSuite
from pathlib import Path


class NavigationTests(TestShellSuite):
    def test_pwd(self):
        """Verify pwd works"""
        self.shell.sendline("type pwd")
        result = self.expect_exact("pwd is a shell builtin")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("pwd")
        cwd = Path.cwd()
        result = self.expect_exact(f"{cwd}")
        self.verify_result(result, "Received expected message")

    def test_cd(self):
        """Verify cd works"""
        self.shell.sendline("type cd")
        result = self.expect_exact("cd is a shell builtin")
        self.verify_result(result, "Received expected message")

        self.prepare_dir("./tmp/nav_test/test")
        self.shell.sendline("cd ./tmp/nav_test/test")
        self.shell.sendline("pwd")
        result = self.expect_exact("./tmp/nav_test/test")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("cd non-existing-dir")
        result = self.expect_exact("cd: non-existing-dir: No such file or directory")
        self.verify_result(result, "Received expected message")

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

        path = "./tmp/nav_test/test"
        self.prepare_dir(path)
        path = Path(path).resolve()
        self.shell.sendline(f"cd {path}")
        self.shell.sendline("pwd")
        result = self.expect_exact(f"{path}")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("cd /non-existing-dir")
        result = self.expect_exact("cd: /non-existing-dir: No such file or directory")
        self.verify_result(result, "Received expected message")

    def test_cd_relative(self):
        """Verify relative file paths"""
        self.prepare_dir("./tmp/nav2/test/dir/dir")
        self.shell.sendline("cd ./tmp/nav2/test")
        self.shell.sendline("pwd")
        result = self.expect_exact("./tmp/nav2/test")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("cd ./dir/dir")
        self.shell.sendline("pwd")
        result = self.expect_exact("./tmp/nav2/test/dir/dir")
        print(self.get_lines())
        self.verify_result(result, "Received expected message")

        self.shell.sendline("cd ../../..")
        self.shell.sendline("pwd")
        result = self.expect_exact("./tmp")
        self.verify_result(result, "Received expected message")

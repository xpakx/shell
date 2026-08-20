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
        path = "./tmp/nav2/test"
        self.shell.sendline(f"cd {path}")
        self.shell.sendline("pwd")
        path = Path(path).resolve()
        result = self.expect_exact(f"{path}")
        self.verify_result(result, "Received expected message")

        path2 = "./dir/dir"
        self.shell.sendline(f"cd {path2}")
        self.shell.sendline("pwd")
        path2 = path / path2
        result = self.expect_exact(f"{path2}")
        self.verify_result(result, "Received expected message")

        path3 = "../../.."
        self.shell.sendline(f"cd {path3}")
        self.shell.sendline("pwd")
        path3 = path2 / path3
        path3 = path3.resolve()
        result = self.expect_exact(f"{path3}")
        self.verify_result(result, "Received expected message")

from base import TestShellSuite, console, PathBuilder, SHELL_PATH
import pexpect
import os


class ExternalOneTests(TestShellSuite):
    def setUp(self):
        console.print(f"\n[cyan]Running:[/cyan]: {self._testMethodName}")
        console.print(f"=> [bold]{self._testMethodDoc}[/bold]")

        new_path = (
                PathBuilder()
                .path("./tmp/test")
                .path("./tmp/test2")
                .path("./tmp/test3")
                .build()
        )
        custom_env = os.environ.copy()
        custom_env["PATH"] = new_path
        self.shell = pexpect.spawn(
                SHELL_PATH,
                encoding="utf-8",
                timeout=1,
                env=custom_env
        )

    def test_type_for_executables(self):
        """Verify type detects executables"""
        self.prepare_file("./tmp/test/my_exe")
        self.prepare_file("./tmp/test2/my_exe")
        self.prepare_file("./tmp/test3/my_exe", True)

        self.shell.sendline("type cat")  # TODO: detect real location first
        result = self.expect_exact("cat is /usr/bin/cat")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("type mkdir")
        result = self.expect_exact("mkdir is /usr/bin/mkdir")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("type my_exe")
        result = self.expect_exact("my_exe is ./tmp/test3/my_exe")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("type invalid_command")
        result = self.expect_exact("invalid_command not found")
        self.verify_result(result, "Received expected message")

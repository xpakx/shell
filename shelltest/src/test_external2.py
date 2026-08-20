from base import TestShellSuite, console, PathBuilder, SHELL_PATH
import pexpect
import os


class ExternalTwoTests(TestShellSuite):
    def setUp(self):
        console.print(f"\n[cyan]Running:[/cyan]: {self._testMethodName}")
        console.print(f"=> [bold]{self._testMethodDoc}[/bold]")

        new_path = (
                PathBuilder()
                .path("./tmp/ext_two")
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

    def test_running_executables(self):
        """Verify executables in PATH run"""
        self.prepare_ext("ext_1.sh", "./tmp/ext_two/exe_000")

        self.shell.sendline("exe_000 test")
        result = self.expect_exact("Number of args passed: 2")
        self.verify_result(result, "Received expected message")
        result = self.expect_exact("#0: exe_000")
        self.verify_result(result, "Received expected message")
        result = self.expect_exact("#1: test")
        self.verify_result(result, "Received expected message")

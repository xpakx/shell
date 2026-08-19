from unittest import TestCase, TestResult, TestLoader, TextTestRunner, TestSuite
from enum import IntEnum
import pexpect
import re
from rich.console import Console
from rich.panel import Panel
from pathlib import Path
from typing import Self
import os

SHELL_PATH = "../target/debug/shell"
console = Console()


class ExpectResult(IntEnum):
    SUCCESS = 0
    TIMEOUT = 1
    EOF = 2


class PathBuilder:
    def __init__(self):
        self.elems = []
        current_path = os.environ.get("PATH", "")
        if current_path:
            self.elems.append(current_path)

    def path(self, path: str) -> Self:
        self.elems.append(path)
        return self

    def build(self) -> str:
        return f"{os.pathsep}".join(reversed(self.elems))


class TestShellSuite(TestCase):
    def setUp(self):
        console.print(f"\n[cyan]Running:[/cyan]: {self._testMethodName}")
        console.print(f"=> [bold]{self._testMethodDoc}[/bold]")

        self.shell = pexpect.spawn(
                SHELL_PATH,
                encoding="utf-8",
                timeout=1
        )

    def tearDown(self):
        if self.shell.isalive():
            self.shell.terminate(force=True)

    def prepare_file(self, name: str, executable: bool = False):
        file_path = Path(name)
        file_path.parent.mkdir(parents=True, exist_ok=True)
        file_path.touch(exist_ok=True)
        if not executable:
            return
        current_perm = file_path.stat().st_mode
        new_perm = current_perm | 0o111
        if new_perm != current_perm:
            file_path.chmod(new_perm)

    def expect_exact(self, text: str) -> ExpectResult:
        index = self.shell.expect_exact([text, pexpect.TIMEOUT, pexpect.EOF])
        return ExpectResult(index)

    def get_lines(self) -> list[str]:
        output = self.shell.before
        ansi_regex = re.compile(
            r"(?:\x1B[@-_]|[\x80-\x9F])[0-?]*[ -/]*[@-~]|\x1b\[\?[0-9]+[hl]"
        )
        clean_output = ansi_regex.sub("", output)
        return clean_output.replace("\r", "").splitlines()

    def verify_result(self, result: ExpectResult, on_succes: str):
        if result == ExpectResult.SUCCESS:
            console.print(f"  [green]{on_succes}[/green]")
        elif result == ExpectResult.TIMEOUT:
            console.print("  [red]timeout[/red]")
            last_line = self.get_lines()[-1].strip()
            console.print(f"  [red]Received: {last_line}[/red]")
            self.fail("timed out")
        elif result == ExpectResult.EOF:
            console.print("  [red]eof[/red]")
            self.fail("shell exited")

    def verify_exits(self, result: ExpectResult, on_succes: str):
        if result == ExpectResult.EOF:
            console.print(f"  [green]{on_succes}[/green]")
        else:
            console.print("  [red]timeout[/red]")
            last_line = self.get_lines()[-1].strip()
            console.print(f"  [red]Received: {last_line}[/red]")
            self.fail("no exit")


class BasicTests(TestShellSuite):
    def test_initial_prompt(self):
        """Verify the shell prints prompt upon starting"""
        result = self.expect_exact("$ ")
        self.verify_result(result, "Initial prompt detected")

    def test_invalid_command_error(self):
        """Verify typing an unknown command outputs the error"""
        self.shell.expect_exact("$ ")
        self.shell.sendline("invalid_command")
        result = self.expect_exact("invalid_command: command not found")
        self.verify_result(result, "Received expected error")

    def test_repl(self):
        """Verify REPL loop works"""
        self.shell.expect_exact("$ ")
        self.shell.sendline("invalid_command_1")
        result = self.expect_exact("invalid_command_1: command not found")
        self.verify_result(result, "Received expected error")
        self.shell.sendline("invalid_command_2")
        result = self.expect_exact("invalid_command_2: command not found")
        self.verify_result(result, "Received expected error")
        self.shell.sendline("invalid_command_3")
        result = self.expect_exact("invalid_command_3: command not found")
        self.verify_result(result, "Received expected error")
        self.shell.sendline("invalid_command_4")
        result = self.expect_exact("invalid_command_4: command not found")
        self.verify_result(result, "Received expected error")
        self.shell.expect_exact("$ ")

    def test_exit(self):
        """Verify exit exits shell"""
        self.shell.sendline("exit")
        result = self.expect_exact("$ exit")
        self.verify_exits(result, "Program exits correctly")
        # TODO no output after exit

    def test_echo(self):
        """Verify exit exits shell"""
        self.shell.sendline("echo test echo")
        result = self.expect_exact("test echo")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("echo test output echo")
        result = self.expect_exact("test output echo")
        self.verify_result(result, "Received expected message")

    def test_type(self):
        """Verify type command works"""
        self.shell.sendline("type echo")
        result = self.expect_exact("echo is a shell builtin")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("type exit")
        result = self.expect_exact("exit is a shell builtin")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("type type")
        result = self.expect_exact("type is a shell builtin")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("type invalid_command")
        result = self.expect_exact("invalid_command not found")
        self.verify_result(result, "Received expected message")

        self.shell.sendline("type invalid_command_2")
        result = self.expect_exact("invalid_command_2 not found")
        self.verify_result(result, "Received expected message")


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


class QuietTestResult(TestResult):
    def printErrors(self):
        pass


class QuietTestRunner(TextTestRunner):
    def __init__(self):
        super().__init__(verbosity=0)

    def _makeResult(self):
        return QuietTestResult(self.stream, self.descriptions, self.verbosity)


def main():
    console.print(
        Panel.fit(
            "[bold magenta]Shell Testing Suite[/bold magenta]",
            border_style="magenta",
            padding=(1, 4),
        )
    )
    runner = QuietTestRunner()
    basic = TestLoader().loadTestsFromTestCase(BasicTests)
    exec1 = TestLoader().loadTestsFromTestCase(ExternalOneTests)
    suite = TestSuite([basic, exec1])
    runner.run(suite)


if __name__ == "__main__":
    main()

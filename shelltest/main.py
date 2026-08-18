from unittest import TestCase, TestResult, TestLoader, TextTestRunner
from enum import IntEnum
import pexpect
import re
from rich.console import Console
from rich.panel import Panel

SHELL_PATH = "../target/debug/shell"
console = Console()


class ExpectResult(IntEnum):
    SUCCESS = 0
    TIMEOUT = 1
    EOF = 2


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
    suite = TestLoader().loadTestsFromTestCase(TestShellSuite)
    runner.run(suite)


if __name__ == "__main__":
    main()

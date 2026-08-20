from unittest import TestCase, TestResult, TextTestRunner
from enum import IntEnum
import pexpect
import re
from rich.console import Console
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

    def prepare_ext(self, script: str, name: str):
        self.prepare_file(name, True)
        script_path = Path("./scripts/") / script
        file_path = Path(name)
        script_content = script_path.read_text()
        file_content = file_path.read_text()
        if file_content != script_content:
            file_path.write_text(script_content)

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


class QuietTestResult(TestResult):
    def printErrors(self):
        pass


class QuietTestRunner(TextTestRunner):
    def __init__(self):
        super().__init__(verbosity=0)

    def _makeResult(self):
        return QuietTestResult(self.stream, self.descriptions, self.verbosity)

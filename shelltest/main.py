from unittest import TestCase, TestResult, TestLoader, TextTestRunner
from functools import wraps
from enum import IntEnum
import pexpect

SHELL_PATH = "../target/debug/shell"


class ExpectResult(IntEnum):
    SUCCESS = 0
    TIMEOUT = 1
    EOF = 2


class TestShellSuite(TestCase):
    def setUp(self):
        print(self._testMethodDoc)
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

    def verify_result(self, result: ExpectResult, on_succes: str):
        if result == ExpectResult.SUCCESS:
            print(on_succes)
        elif result == ExpectResult.TIMEOUT:
            print("timeout")
            self.fail("timed out")
        elif result == ExpectResult.EOF:
            print("eof")
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
    print("shell testing suite")
    runner = QuietTestRunner()
    suite = TestLoader().loadTestsFromTestCase(TestShellSuite)
    runner.run(suite)


if __name__ == "__main__":
    main()

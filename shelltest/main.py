from unittest import TestCase, TestResult, TestLoader, TextTestRunner
from functools import wraps
import pexpect

SHELL_PATH = "../target/debug/shell"


def ensure_exceptions(func):
    @wraps(func)
    def wrapper(self, *args, **kwargs):
        try:
            return func(self, *args, **kwargs)
        except pexpect.TIMEOUT:
            print("timeout")
            self.fail("timed out")
        except pexpect.EOF:
            print("eof")
            self.fail("shell exited")
    return wrapper


class TestShellSuite(TestCase):
    def setUp(self):
        print(self._testMethodName)
        self.shell = pexpect.spawn(
                SHELL_PATH,
                encoding="utf-8",
                timeout=1
        )

    def tearDown(self):
        if self.shell.isalive():
            self.shell.terminate(force=True)

    @ensure_exceptions
    def test_suite(self):
        """Verify the shell prints prompt upon starting"""
        self.shell.expect_exact("$ ")
        print("Initial prompt detected")


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

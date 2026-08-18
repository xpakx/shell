from unittest import TestCase, TestResult, TestLoader, TextTestRunner
import pexpect

SHELL_PATH = "../target/debug/shell"


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

    def test_suite(self):
        """Verify the suite starts shell"""
        try:
            self.shell.expect_exact("$ ")
            print("Initial prompt detected")
        except pexpect.TIMEOUT:
            print("timeout")
            self.fail("timed out")
        except pexpect.EOF:
            print("eof")
            self.fail("shell exited")


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

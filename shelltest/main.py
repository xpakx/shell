from unittest import TestCase
from unittest import main as test_main
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


def main():
    print("shell testing suite")
    test_main()


if __name__ == "__main__":
    main()

from unittest import TestLoader, TestSuite
from rich.panel import Panel
from base import console, QuietTestRunner
from test_basic import BasicTests
from test_external import ExternalOneTests
from test_external2 import ExternalTwoTests
from test_navigation import NavigationTests


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
    exec1 = TestLoader().loadTestsFromTestCase(ExternalTwoTests)
    exec1 = TestLoader().loadTestsFromTestCase(NavigationTests)
    suite = TestSuite([basic, exec1])
    runner.run(suite)


if __name__ == "__main__":
    main()

from base import TestShellSuite


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

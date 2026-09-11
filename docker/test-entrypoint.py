#!/usr/bin/env python3
"""Exercise the shipped entrypoint without containers or a node database."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class EntrypointTest(unittest.TestCase):
    def run_entrypoint(self, variables=None, args=(), success=True):
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            mock = directory / 'duniter'
            mock.write_text('''#!/usr/bin/env python3
import json, os, sys
with open(os.environ['TEST_CALLS'], 'a') as output:
    output.write(json.dumps(sys.argv[1:]) + '\\n')
if sys.argv[1:3] == ['key', 'inspect-node-key']:
    print('test-peer')
''')
            mock.chmod(0o755)
            env = {k: v for k, v in os.environ.items() if not k.startswith('DUNITER_')}
            env.update(variables or {})
            env.update(PATH=str(directory) + os.pathsep + env['PATH'], TEST_CALLS=str(directory / 'calls'))
            result = subprocess.run(['/bin/bash', str(ROOT / 'docker/docker-entrypoint'), *args],
                                    env=env, capture_output=True, text=True)
            calls = [json.loads(line) for line in (directory / 'calls').read_text().splitlines()] if (directory / 'calls').exists() else []
            if success:
                self.assertEqual(result.returncode, 0, result.stderr)
            else:
                self.assertNotEqual(result.returncode, 0)
            return calls, result

    def test_legacy_defaults_and_identity(self):
        calls, _ = self.run_entrypoint()
        self.assertEqual(calls[0][:2], ['key', 'generate-node-key'])
        self.assertEqual(calls[1][:2], ['key', 'inspect-node-key'])
        for arg in ['--dev', '--base-path=/var/lib/duniter', '--rpc-external', '--rpc-cors=all',
                    '--node-key-file=/var/lib/duniter/node.key']:
            self.assertIn(arg, calls[-1])

    def test_legacy_aliases_and_profiles(self):
        calls, _ = self.run_entrypoint({'DUNITER_CHAIN_NAME': 'g1', 'DUNITER_NODE_NAME': 'old name',
            'DUNITER_VALIDATOR': 'YES', 'DUNITER_DISABLE_TELEMETRY': '1',
            'DUNITER_DISABLE_PROMETHEUS': 'true', 'DUNITER_PRUNING_PROFILE': 'archive'})
        for arg in ['--chain=g1', '--name=old name', '--validator', '--rpc-methods=Unsafe',
                    '--no-telemetry', '--no-prometheus', '--blocks-pruning=archive', '--state-pruning=archive']:
            self.assertIn(arg, calls[-1])

    def test_new_variables_override_legacy(self):
        calls, _ = self.run_entrypoint({'DUNITER_CHAIN_NAME': 'g1', 'DUNITER_CHAIN': 'gtest',
            'DUNITER_NODE_NAME': 'old', 'DUNITER_NAME': 'new', 'DUNITER_PRUNING_PROFILE': 'light',
            'DUNITER_BLOCKS_PRUNING': 'archive', 'DUNITER_DISABLE_TELEMETRY': 'true',
            'DUNITER_NO_TELEMETRY': 'false', 'DUNITER_NODE_KEY_FILE': '/custom/key'})
        self.assertEqual(len(calls), 1)
        for arg in ['--chain=gtest', '--name=new', '--blocks-pruning=archive', '--node-key-file=/custom/key']:
            self.assertIn(arg, calls[0])
        self.assertNotIn('--no-telemetry', calls[0])

    def test_cli_overrides_environment(self):
        calls, _ = self.run_entrypoint({'DUNITER_NAME': 'env', 'DUNITER_BLOCKS_PRUNING': 'archive'},
            ['--chain', 'g1', '--name=cli', '--blocks-pruning', '512', '-d', '/custom', '--node-key=secret'])
        self.assertEqual(len(calls), 1)
        for arg in ['--name=env', '--blocks-pruning=archive', '--dev', '--base-path=/var/lib/duniter']:
            self.assertNotIn(arg, calls[0])
        self.assertIn('512', calls[0])

    def test_raw_mode_ignores_environment(self):
        calls, _ = self.run_entrypoint({'DUNITER_NO_TELEMETRY': 'invalid'}, ['--', 'key', 'inspect'])
        self.assertEqual(calls, [['key', 'inspect']])

    def test_repeated_values_are_literal_and_not_logged(self):
        value = '$(exit 99); `exit 99` "quoted"'
        calls, result = self.run_entrypoint({'DUNITER_NODE_KEY': 'private', 'DUNITER_NAME': value,
            'DUNITER_TELEMETRY_URL': 'wss://one.example 0\nwss://two.example 1'})
        self.assertIn('--name=' + value, calls[-1])
        self.assertIn('--telemetry-url=wss://one.example 0', calls[-1])
        self.assertIn('--telemetry-url=wss://two.example 1', calls[-1])
        self.assertNotIn('private', result.stdout)
        self.assertNotIn(value, result.stdout)

    def test_local_profile(self):
        calls, _ = self.run_entrypoint({'DUNITER_CHAIN_NAME': 'gtest_local'})
        self.assertNotIn('--base-path=/var/lib/duniter', calls[-1])
        for arg in ['--validator', '--sealing=manual', '--tmp', '--unsafe-force-node-key-generation']:
            self.assertIn(arg, calls[-1])

    def test_invalid_boolean_and_empty_list_item(self):
        for values in [{'DUNITER_NO_TELEMETRY': 'invalid'}, {'DUNITER_BOOTNODES': 'one\n\ntwo'}]:
            calls, _ = self.run_entrypoint(values, success=False)
            self.assertEqual(calls, [])


if __name__ == '__main__':
    unittest.main()

#!/usr/bin/python3

import shutil
import os
import subprocess
from pathlib import Path
import json
import sys
import argparse

os.chdir(Path(__file__).resolve().parent.parent)

MODIFIERS = {
    "amd": "emulated AMD CPU, no KVM (slow)",
    "bios": "boot with BIOS instead of UEFI",
}

parser = argparse.ArgumentParser(description="Run all tests, or only <test_name>.")
parser.add_argument("test_name", nargs="?", help="run only this test")
parser.add_argument("--debug", action="store_true", help="attach gdb (single test only)")
for flag, help_text in MODIFIERS.items():
    parser.add_argument(f"--{flag}", action="store_true", help="Modifier: "+help_text)
args = parser.parse_args()

if args.debug and args.test_name is None:
    parser.error("--debug needs a test name")

start_args = [f"--{f}" for f in MODIFIERS if getattr(args, f)] + (["--debug"] if args.debug else [])

def print_result(name, code):
    print(f"{name}: ", end="")
    if code == 33:
        print("\033[92mSUCCESS\033[0m")
        return True
    elif code == 35:
        print("\033[91mFAIL\033[0m")
    elif code == 124:
        print("\033[93mTIMEOUT\033[0m")
    elif code == 0:
        print("\033[93mTRIPLE FAULT\033[0m")
    else:
        print(f"\033[93mUNKNOWN (exit code {code})\033[0m")
    return False

LOG_DIR="target/test-logs"

def run_test(name, executable, debug):
    result = subprocess.run(["scripts/start.sh", executable, *start_args], capture_output=not debug, text=True)
    log_path = Path(LOG_DIR) / f"{name}.log"
    if not debug:
        with open(log_path, "w") as f:
            f.write(f"--- STDOUT ---\n\n")
            f.write(result.stdout)
            f.write(f"\n\n--- STDERR ---\n\n")
            f.write(result.stderr)
    return result

if args.test_name is None:
    shutil.rmtree(Path(LOG_DIR), ignore_errors=True)
Path(LOG_DIR).mkdir(parents=True, exist_ok=True)

result = subprocess.run(["cargo", "test", "--no-run", "--message-format=json"], stdout=subprocess.PIPE, text=True)
if result.returncode != 0:
    sys.exit(1)


success = 0
total = 0
print()

for line in result.stdout.splitlines():
    data = json.loads(line)
    if "target" not in data or "executable" not in data or data["target"]["kind"] != ["test"]:
        continue  # Skip non-target lines
    name = data["target"]["name"]
    executable = data["executable"]
    if executable is None:
        continue  # Skip non-executable targets

    if args.test_name is not None and name != args.test_name:
        continue  # Skip tests that don't match the specified test name
    
    total += 1

    result_test = run_test(name, executable, args.debug)
    is_success = print_result(name, result_test.returncode)
    if is_success:
        success += 1

if args.test_name is not None and total == 0:
    print(f"No test found with the name '{args.test_name}'.")
    sys.exit(1)
elif args.test_name is None:
    print(f"\n{success}/{total} tests passed.")
    print(f"Logs can be found in '{LOG_DIR}/'.")
    print(f"To rerun a specific test, use: 'scripts/tests.py <test_name>'")
    if success != total:
        sys.exit(1)
else:
    print(f"\nLog for test '{args.test_name}' can be found in '{LOG_DIR}/{args.test_name}.log'.")
    if success == 1:
        sys.exit(0)
    else:
        sys.exit(1)
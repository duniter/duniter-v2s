#!/usr/bin/env python3
"""Keep cargo-auditable instrumentation out of nested Wasm compilations."""

import os
from pathlib import Path
import sys


def compiler_command(args, environ):
    command = list(args)
    workspace_wrapper = environ.get("RUSTC_WORKSPACE_WRAPPER")
    # Cargo nests wrappers as: RUSTC_WRAPPER RUSTC_WORKSPACE_WRAPPER rustc args.
    # Only remove cargo-auditable, never the compiler or an unrelated wrapper.
    if (
        workspace_wrapper
        and command
        and Path(command[0]).resolve() == Path(workspace_wrapper).resolve()
        and Path(workspace_wrapper).name == "cargo-auditable"
    ):
        rustc_args = command[2:]
        target = None
        for index, arg in enumerate(rustc_args):
            if arg == "--target" and index + 1 < len(rustc_args):
                target = rustc_args[index + 1]
            elif arg.startswith("--target="):
                target = arg.split("=", 1)[1]
        if target and target.split("-", 1)[0] in ("wasm32", "wasm32v1", "wasm64"):
            command = command[1:]

    previous = environ.get("DUNITER_AUDIT_ORIGINAL_RUSTC_WRAPPER")
    if previous:
        command.insert(0, previous)
    return command


if __name__ == "__main__":
    command = compiler_command(sys.argv[1:], os.environ)
    os.execvp(command[0], command)

#!/usr/bin/env python3
import argparse
import json
from collections import OrderedDict
from pathlib import Path
import shutil
import sys


def transform(data: OrderedDict) -> (OrderedDict, bool):
    """
    Apply the migration rules on a single JSON object, preserving order and
    all other properties. Returns (new_data, changed_bool).
    """
    had_check = "check_steps" in data
    had_run = "run_step" in data
    has_setup = "setup_steps" in data
    has_test = "test_steps" in data

    changed = False
    out = OrderedDict()

    for k, v in data.items():
        if k == "check_steps":
            if not has_setup:
                out["setup_steps"] = v
            # Drop legacy key either way
            changed = True
            continue

        if k == "run_step":
            if not has_test:
                if isinstance(v, list):
                    out["test_steps"] = v
                else:
                    out["test_steps"] = [v]
            # Drop legacy key either way
            changed = True
            continue

        # passthrough
        out[k] = v

    return out, changed or had_check or had_run  # be explicit


def migrate_one(config_path: Path, overwrite_steps: bool, make_backup: bool) -> bool:
    """
    Migrate a single config.json file and rename it to steps.json.
    Returns True if any file changes were made.
    """
    # Parse JSON preserving key order
    try:
        data = json.loads(
            config_path.read_text(encoding="utf-8"), object_pairs_hook=OrderedDict
        )
    except Exception as e:
        print(f"  !! Failed to parse JSON: {e}")
        return False

    new_data, changed = transform(data)

    steps_path = config_path.with_name("steps.json")

    # Backup original config.json
    try:
        if make_backup:
            backup = config_path.with_suffix(config_path.suffix + ".bak")
            shutil.copy2(config_path, backup)
            print(f"  .. Backup written: {backup}")
    except Exception as e:
        print(f"  !! Failed to create backup: {e}")
        return False

    # If steps.json exists and not overwriting, skip safely
    if steps_path.exists() and not overwrite_steps:
        print(
            f"  !! {steps_path} already exists. Skipping (use --overwrite to replace)."
        )
        return False

    # If no JSON changes are needed, we still perform the rename to steps.json
    # (write the current/new content into steps.json to ensure canonical naming).
    try:
        steps_path.write_text(
            json.dumps(new_data, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        print(f"  .. Wrote migrated JSON → {steps_path}")
    except Exception as e:
        print(f"  !! Failed to write {steps_path}: {e}")
        return False

    # Remove the old config.json (we've got a backup)
    try:
        config_path.unlink()
        print(f"  .. Removed legacy file: {config_path}")
    except Exception as e:
        print(f"  !! Failed to remove {config_path}: {e}")
        # We still consider migration successful because steps.json is written
        # but signal partial success.
        return True

    return True


def main():
    ap = argparse.ArgumentParser(
        description=(
            "Migrate config.json files: "
            "check_steps→setup_steps, run_step→test_steps[], "
            "then rename config.json → steps.json. "
            "All other properties are preserved."
        )
    )
    ap.add_argument(
        "root",
        nargs="?",
        default=".",
        help="Root directory to scan (default: current dir)",
    )
    ap.add_argument(
        "--no-backup", action="store_true", help="Do not write config.json.bak backups"
    )
    ap.add_argument(
        "--overwrite",
        action="store_true",
        help="Overwrite steps.json if it already exists",
    )
    args = ap.parse_args()

    root = Path(args.root).resolve()
    files = sorted(root.rglob("config.json"))
    if not files:
        print(f"No config.json files under {root}")
        return 0

    print(f"Found {len(files)} config.json file(s) under {root}.\n")

    total_changed = 0
    for i, path in enumerate(files, 1):
        print(f"[{i}/{len(files)}] {path}")
        resp = (
            input("    Press Enter to migrate & rename, 's' to skip, or 'q' to quit: ")
            .strip()
            .lower()
        )
        if resp == "q":
            print("Aborting.")
            break
        if resp == "s":
            print("  .. Skipped.\n")
            continue

        changed = migrate_one(
            path, overwrite_steps=args.overwrite, make_backup=not args.no_backup
        )
        total_changed += int(changed)
        print()

    print(f"Done. Modified/Renamed {total_changed} file(s).")
    return 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except KeyboardInterrupt:
        print("\nInterrupted.")
        sys.exit(130)

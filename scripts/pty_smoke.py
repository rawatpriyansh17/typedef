#!/usr/bin/env python3
"""Exercise the real binary with a Unix PTY. Uses only Python's standard library."""
import fcntl
import json
import os
from pathlib import Path
import pty
import select
import struct
import subprocess
import tempfile
import termios
import time

BINARY = os.environ.get("TYPEDEF_BINARY", str(Path(__file__).resolve().parents[1] / "target/release/typedef"))


def read_for(fd, seconds):
    data = b""
    end = time.monotonic() + seconds
    while time.monotonic() < end:
        ready, _, _ = select.select([fd], [], [], min(0.05, max(0, end-time.monotonic())))
        if ready:
            try:
                data += os.read(fd, 65536)
            except OSError:
                break
    return data


def launch(args, profile):
    master, slave = pty.openpty()
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 32, 110, 0, 0))
    before = termios.tcgetattr(slave)
    process = subprocess.Popen([BINARY, "--data-dir", str(profile), *args], stdin=slave, stdout=slave, stderr=slave, env={**os.environ, "TERM": "xterm-256color"})
    output = b""
    deadline = time.monotonic() + 3
    while b"typedef" not in output and process.poll() is None and time.monotonic() < deadline:
        output += read_for(master, 0.1)
    if b"typedef" not in output:
        process.terminate()
        process.wait(timeout=3)
        os.close(master)
        os.close(slave)
        raise AssertionError(output[-1500:])
    return process, master, slave, before


def finish(process, master, slave, before):
    os.write(master, b"\x03")
    output = read_for(master, 0.3)
    assert b"\x1b[?1006l" in output, "mouse capture was not disabled"
    assert b"\x1b[?2004l" in output, "bracketed paste was not disabled"
    assert process.wait(timeout=3) == 0
    assert termios.tcgetattr(slave) == before, "terminal mode was not restored"
    os.close(master)
    os.close(slave)


with tempfile.TemporaryDirectory(prefix="typedef-smoke-") as directory:
    root = Path(directory)
    sample = root / "sample.txt"
    sample.write_text("hello world")
    profile = root / "profile"
    process, master, slave, before = launch(["--file", str(sample)], profile)
    try:
        os.write(master, b"\r")
        read_for(master, 0.15)
        os.write(master, b"\x1b[200~PASTED\x1b[201~")
        read_for(master, 0.1)
        # Mistake, correction, then a successful run.
        os.write(master, b"x\x7f")
        for c in b"hello world":
            os.write(master, bytes([c]))
            read_for(master, 0.035)
        output = read_for(master, 0.25)
        record = json.loads((profile / "profile.json").read_text())["history"]
        assert len(record) == 1
        assert abs(record[0]["accuracy"] - 100*11/12) < 0.001
        assert not record[0]["failed"]
        assert record[0]["wpm"] > 0
        # Resize while on results and return home; no panic or duplicate record.
        fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", 12, 35, 0, 0))
        import signal
        process.send_signal(signal.SIGWINCH)
        read_for(master, 0.1)
    finally:
        finish(process, master, slave, before)
    # Exercise grouped settings, live theme changes and all profile tabs.
    process, master, slave, before = launch([], profile)
    try:
        os.write(master, b"s\t")  # appearance / theme
        read_for(master, 0.15)
        os.write(master, b"\x1b[C")
        read_for(master, 0.15)
        saved = json.loads((profile / "profile.json").read_text())
        assert saved["settings"]["theme"] == 1
        assert "sound" not in saved["settings"] and "volume" not in saved["settings"]
        os.write(master, b"\x1b")
        read_for(master, 0.15)
        os.write(master, b"t\x1b[C")
        read_for(master, 0.15)
        assert json.loads((profile / "profile.json").read_text())["settings"]["theme"] == 1
        os.write(master, b"\r")
        read_for(master, 0.15)
        assert json.loads((profile / "profile.json").read_text())["settings"]["theme"] == 2
        os.write(master, b"\x1b[C\x1b")  # preview another palette, then cancel
        read_for(master, 0.15)
        assert json.loads((profile / "profile.json").read_text())["settings"]["theme"] == 2
        os.write(master, b"p\t\t\t")
        output = read_for(master, 0.2)
        assert process.poll() is None
        assert b"key heatmap" in output or b"untested" in output
        # No Home detour: profile -> settings -> themes -> journey.
        os.write(master, b"s")
        assert b"CUSTOMIZE" in read_for(master, 0.2)
        os.write(master, b"t\x1b[F\r")  # End, apply the last of 28 themes
        read_for(master, 0.2)
        assert json.loads((profile / "profile.json").read_text())["settings"]["theme"] == 27
        os.write(master, b"g")
        assert b"JOURNEY" in read_for(master, 0.2)
        # The settings header is at column 45, row 2 in this 110-column viewport.
        os.write(master, b"\x1b[<0;46;2M\x1b[<0;46;2m")
        assert b"CUSTOMIZE" in read_for(master, 0.2)
        os.write(master, b"b")
        read_for(master, 0.15)
        os.write(master, b"pst")  # These must be literal key-lab input.
        output = read_for(master, 0.15)
        assert b"Char" in output
        os.write(master, b"\x1b[15~")  # F5 can leave the key lab directly.
        assert b"PREVIEW" in read_for(master, 0.2)
        os.write(master, b"s")
        read_for(master, 0.15)
        os.write(master, b"\x1b[C")  # appearance/theme wraps from 27 to 0
        read_for(master, 0.15)
        assert json.loads((profile / "profile.json").read_text())["settings"]["theme"] == 0
    finally:
        finish(process, master, slave, before)
    exported = json.loads(subprocess.check_output([BINARY, "--data-dir", str(profile), "--export"]))
    assert len(exported["history"]) == 1
    # Load a new file while the app is running. Paste is a path only here;
    # printable navigation letters in filenames must not switch screens.
    sample = root / "custom typing sample.txt"
    sample.write_text("    x\n  y")
    imported = root / "imported"
    process, master, slave, before = launch([], imported)
    try:
        os.write(master, b"o")
        read_for(master, 0.15)
        os.write(master, b"\x1b[200~" + str(sample).encode() + b"\x1b[201~")
        read_for(master, 0.15)
        os.write(master, b"\r")
        read_for(master, 0.15)
        assert (imported / "import.txt").read_text() == "    x\n  y"
        for key in [b"\t", b"x", b"\r", b"\t", b"y"]:
            os.write(master, key)
            read_for(master, 0.06)
        record = json.loads((imported / "profile.json").read_text())["history"]
        assert len(record) == 1 and record[0]["accuracy"] == 100
        assert not record[0]["failed"]
    finally:
        finish(process, master, slave, before)
    # Both binaries share implementation, profile and CLI behavior.
    alias = str(Path(BINARY).with_name("tpdf"))
    assert subprocess.check_output([alias, "--version"]) == subprocess.check_output([BINARY, "--version"])
    alias_history = json.loads(subprocess.check_output([alias, "--data-dir", str(imported), "--export"]))["history"]
    canonical_history = json.loads(subprocess.check_output([BINARY, "--data-dir", str(imported), "--export"]))["history"]
    assert alias_history == canonical_history
    assert len(alias_history) == 1 and alias_history[0]["accuracy"] == 100
    for mode in ["endless", "zen"]:
        continuous = root / mode
        process, master, slave, before = launch(["--mode", mode, "--words", "1"], continuous)
        try:
            os.write(master, b"\r")
            read_for(master, 0.1)
            os.write(master, b"a b c d e ")
            read_for(master, 0.2)
            os.write(master, b"\x04")
            read_for(master, 0.2)
            history = json.loads((continuous / "profile.json").read_text())["history"]
            assert len(history) == 1 and not history[0]["failed"]
        finally:
            finish(process, master, slave, before)
    process, master, slave, before = launch(["--mode", "time", "--seconds", "1"], root / "timed")
    try:
        os.write(master, b"\r")
        read_for(master, 0.1)
        os.write(master, b"a")
        # Wait for the observable save, not an assumed CI scheduling latency.
        deadline = time.monotonic() + 5
        while not (root / "timed/profile.json").exists() and time.monotonic() < deadline:
            read_for(master, 0.1)
        record = json.loads((root / "timed/profile.json").read_text())["history"]
        assert len(record) == 1 and record[0]["seconds"] == 1.0
    finally:
        finish(process, master, slave, before)
print("PASS: real terminal input, correction, paste rejection, live file loading, Tab indentation, tpdf alias, Endless/Zen, resize, timed deadline, export, settings, 28 themes, direct/mouse navigation, key-lab routing, profile tabs and terminal restoration")

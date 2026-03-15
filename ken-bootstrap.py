#!/usr/bin/env python3
"""
ken-bootstrap.py - Minimal viable ken wake implementation

Spawns a Claude Code agent in a git worktree, walks it through kenning
frames for context, then delivers a task. The agent does real work
(edits files, runs commands) and you get a worktree back with changes.

Usage:
    python ken-bootstrap.py <ken-path> --task "your task here"

Example:
    python ken-bootstrap.py cli/wake --task "implement the frame parser"

Requirements:
    claude CLI installed and authenticated
"""

import argparse
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path


# ============================================================================
# Data Structures
# ============================================================================

@dataclass
class Frame:
    number: int
    title: str
    prompt: str


@dataclass
class Kenning:
    path: str
    frames: list[Frame]


# ============================================================================
# Kenning Parser
# ============================================================================

def parse_kenning(filepath: str) -> Kenning:
    """Parse a kenning.md file into structured frames."""

    with open(filepath, 'r') as f:
        content = f.read()

    # Extract ken path from first header
    path_match = re.search(r'^# Kenning:\s*(.+)$', content, re.MULTILINE)
    path = path_match.group(1).strip() if path_match else "unknown"

    # Find all frames
    frame_pattern = r'## Frame (\d+):\s*(.+?)\n(.*?)(?=## Frame \d+:|## Task|## Reflection|## Meta|\Z)'
    matches = re.findall(frame_pattern, content, re.DOTALL)

    frames = []
    for num, title, prompt in matches:
        prompt = prompt.strip()
        prompt = re.sub(r'^---+\s*$', '', prompt, flags=re.MULTILINE).strip()

        frames.append(Frame(
            number=int(num),
            title=title.strip(),
            prompt=prompt
        ))

    frames.sort(key=lambda f: f.number)

    return Kenning(path=path, frames=frames)


# ============================================================================
# Prompt Composition
# ============================================================================

def compose_prompt(kenning: Kenning, task: str) -> str:
    """Compose a single prompt from kenning frames + task.

    This flattens multi-turn frame walking into a single prompt. The original
    design had each frame as a separate conversation turn so the agent's own
    responses accumulated in context. We sacrifice that for Claude Code's tool
    access (file editing, command execution) which requires -p mode. The agent
    is instructed to think through frames sequentially but enforcement is
    structural, not mechanical.
    """

    parts = []

    parts.append(
        "You are being woken by ken. You will first walk through a series of "
        "frames that build your understanding of this area of the codebase, "
        "then complete a task.\n\n"
        "Think through each frame carefully before moving to the next.\n"
    )

    parts.append("---\n")

    for frame in kenning.frames:
        parts.append(f"## Frame {frame.number}: {frame.title}\n")
        parts.append(f"{frame.prompt}\n")

    parts.append("---\n")

    parts.append(f"## Task\n")
    parts.append(f"{task}\n\n")
    parts.append("Complete this task by making the necessary changes to the codebase.")

    parts.append("\n---\n")
    parts.append("## Reflection\n")
    parts.append(
        "After completing the task, create a file called `reflection.md` in the "
        "repo root with your reflection on this session:\n\n"
        "1. Did the frames prepare you well? What was clear, what was missing?\n"
        "2. What did you discover during work that future agents should know?\n"
        "3. If you could improve the kenning, what would you change?\n\n"
        "Be specific. Your reflection helps future agents woken into this ken."
    )

    return "\n".join(parts)


# ============================================================================
# Claude CLI Execution
# ============================================================================

def run_claude(prompt: str, verbose: bool = True) -> subprocess.CompletedProcess:
    """Spawn claude CLI in a worktree with the composed prompt."""

    if not shutil.which("claude"):
        print("Error: claude CLI not found on PATH")
        print("Install: https://docs.anthropic.com/en/docs/claude-code")
        sys.exit(1)

    cmd = [
        "claude",
        "-p",
        "--worktree",
        "--output-format", "text",
    ]

    if verbose:
        print("[Spawning claude agent in worktree...]\n")

    result = subprocess.run(
        cmd,
        input=prompt,
        stderr=subprocess.PIPE,
        text=True,
    )

    return result


# ============================================================================
# Reflection Persistence
# ============================================================================

def _list_worktrees() -> set[Path]:
    """Return the set of current git worktree paths."""
    try:
        result = subprocess.run(
            ["git", "worktree", "list", "--porcelain"],
            capture_output=True, text=True, check=True,
        )
    except subprocess.CalledProcessError:
        return set()

    paths = set()
    for line in result.stdout.splitlines():
        if line.startswith("worktree "):
            paths.add(Path(line.split(" ", 1)[1]))
    return paths


def save_reflection(ken_path: str, task: str, new_worktrees: set[Path]):
    """Persist the reflection from the worktree created by this session.

    Only considers worktrees in new_worktrees (those created after we
    snapshotted the worktree list before spawning claude). This prevents
    silently saving a stale reflection left behind by a prior session.
    """

    worktree_path = None
    for candidate in new_worktrees:
        if (candidate / "reflection.md").exists():
            worktree_path = candidate
            break

    if not worktree_path:
        return

    reflection_src = worktree_path / "reflection.md"
    reflection_text = reflection_src.read_text()

    # Save to reflections directory
    reflection_dir = Path("reflections") / ken_path
    reflection_dir.mkdir(parents=True, exist_ok=True)

    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    reflection_dst = reflection_dir / f"{timestamp}.md"

    header = (
        f"# Reflection: {ken_path}\n\n"
        f"**Timestamp**: {datetime.now().isoformat()}\n"
        f"**Task**: {task}\n\n"
        f"---\n\n"
    )
    reflection_dst.write_text(header + reflection_text)
    print(f"Reflection saved to: {reflection_dst}")


# ============================================================================
# Main
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Ken Wake - Spawn an agent with structured context in a worktree",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Example:
    python ken-bootstrap.py cli/wake --task "implement the parser"

The script will:
1. Load the kenning from kens/<path>/kenning.md
2. Compose frames + task into a structured prompt
3. Spawn a claude agent in a git worktree
4. The agent does real work (edits files, runs commands)
5. You review changes in the worktree
        """
    )

    parser.add_argument("ken_path", help="Path to ken (e.g., 'core/kenning-parser')")
    parser.add_argument("--task", "-t", required=True, help="Task to accomplish")
    parser.add_argument("--kens-dir", default="kens", help="Directory containing kens")
    parser.add_argument("--quiet", "-q", action="store_true", help="Minimal output")

    args = parser.parse_args()

    verbose = not args.quiet

    # Find kenning file
    kenning_path = Path(args.kens_dir) / args.ken_path / "kenning.md"

    if not kenning_path.exists():
        print(f"Error: Kenning not found: {kenning_path}")
        sys.exit(1)

    # Parse kenning
    kenning = parse_kenning(str(kenning_path))

    if not kenning.frames:
        print("Error: No frames found in kenning")
        sys.exit(1)

    if verbose:
        print(f"Ken Wake: {args.ken_path}")
        print(f"Task: {args.task}")
        print(f"Kenning: {kenning_path}")
        print(f"Frames: {len(kenning.frames)}")
        print()

    # Compose prompt from frames + task
    prompt = compose_prompt(kenning, args.task)

    # Snapshot worktrees so we can identify the one claude creates
    worktrees_before = _list_worktrees()

    # Run claude in a worktree
    result = run_claude(prompt, verbose)

    if result.stderr:
        print(result.stderr, file=sys.stderr)

    if result.returncode != 0:
        print(f"\nAgent exited with code {result.returncode}")
        sys.exit(result.returncode)

    # Collect reflection only from the worktree created by this session
    new_worktrees = _list_worktrees() - worktrees_before
    save_reflection(args.ken_path, args.task, new_worktrees)

    if verbose:
        print("\n" + "=" * 60)
        print("Session complete. Check the worktree for changes.")
        print("Run 'git worktree list' to find it.")
        print("=" * 60)


if __name__ == "__main__":
    main()

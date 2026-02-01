#!/usr/bin/env python3
"""
ken-bootstrap.py - Minimal viable ken wake implementation

This is the bootstrap script. It does just enough to test if kennings work.
Once we validate the concept, we build the real thing using this.

Usage:
    python ken-bootstrap.py <ken-path> --task "your task here"
    
Example:
    python ken-bootstrap.py core/kenning-parser --task "implement the frame parser"

Requirements:
    pip install anthropic
    export ANTHROPIC_API_KEY=your-key
"""

import argparse
import os
import re
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
# Kenning Parser (Minimal)
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
        # Clean up the prompt
        prompt = prompt.strip()
        # Remove horizontal rules
        prompt = re.sub(r'^---+\s*$', '', prompt, flags=re.MULTILINE).strip()
        
        frames.append(Frame(
            number=int(num),
            title=title.strip(),
            prompt=prompt
        ))
    
    # Sort by frame number
    frames.sort(key=lambda f: f.number)
    
    return Kenning(path=path, frames=frames)


# ============================================================================
# Agent Communication
# ============================================================================

def create_client():
    """Create Anthropic client."""
    try:
        import anthropic
    except ImportError:
        print("Error: anthropic package not installed")
        print("Run: pip install anthropic")
        sys.exit(1)
    
    api_key = os.environ.get('ANTHROPIC_API_KEY')
    if not api_key:
        print("Error: ANTHROPIC_API_KEY environment variable not set")
        sys.exit(1)
    
    return anthropic.Anthropic(api_key=api_key)


def walk_frames(client, kenning: Kenning, verbose: bool = True) -> list[dict]:
    """Walk through kenning frames, building context."""
    
    messages = []
    
    for frame in kenning.frames:
        if verbose:
            print(f"\n{'='*60}")
            print(f"Frame {frame.number}: {frame.title}")
            print('='*60)
            print(f"\n[Sending prompt...]\n")
        
        # Add frame as user message
        messages.append({
            "role": "user",
            "content": frame.prompt
        })
        
        # Get response
        response = client.messages.create(
            model="claude-sonnet-4-20250514",
            max_tokens=4096,
            messages=messages
        )
        
        assistant_message = response.content[0].text
        
        # Add response to context
        messages.append({
            "role": "assistant", 
            "content": assistant_message
        })
        
        if verbose:
            print(assistant_message)
            print(f"\n[Frame {frame.number} complete]")
    
    return messages


def deliver_task(client, messages: list[dict], task: str, verbose: bool = True) -> list[dict]:
    """Deliver the task and get work response."""
    
    if verbose:
        print(f"\n{'='*60}")
        print("TASK")
        print('='*60)
        print(f"\n{task}\n")
    
    task_prompt = f"""## Task

{task}

Please complete this task based on the understanding you've built through our conversation.
Provide your implementation, explanation, or solution."""

    messages.append({
        "role": "user",
        "content": task_prompt
    })
    
    response = client.messages.create(
        model="claude-sonnet-4-20250514",
        max_tokens=8192,
        messages=messages
    )
    
    work_response = response.content[0].text
    
    messages.append({
        "role": "assistant",
        "content": work_response
    })
    
    if verbose:
        print("\n[Agent working...]\n")
        print(work_response)
    
    return messages


def collect_reflection(client, messages: list[dict], verbose: bool = True) -> str:
    """Prompt for and collect reflection."""
    
    if verbose:
        print(f"\n{'='*60}")
        print("REFLECTION")
        print('='*60)
    
    reflection_prompt = """## Reflection

Your work session is complete. Before this context clears, please reflect:

1. **Preparation**: Did the frames prepare you well for this task? What was clear from the start? What did you wish you'd understood better going in?

2. **Gaps**: Were there moments during the work where you needed context you didn't have? What was missing?

3. **Discoveries**: What did you figure out during this work that future agents working in this area should know from the start?

4. **Kenning Improvements**: If you could add, remove, or modify frames in the preparation sequence, what would you change? Be specific.

Your reflection will be used to improve how future instances are prepared for this work."""

    messages.append({
        "role": "user",
        "content": reflection_prompt
    })
    
    response = client.messages.create(
        model="claude-sonnet-4-20250514",
        max_tokens=4096,
        messages=messages
    )
    
    reflection = response.content[0].text
    
    if verbose:
        print(f"\n{reflection}")
    
    return reflection


def save_reflection(ken_path: str, task: str, reflection: str, base_dir: str = "."):
    """Save reflection to file."""
    
    # Create reflections directory
    reflection_dir = Path(base_dir) / "reflections" / ken_path.replace("/", os.sep)
    reflection_dir.mkdir(parents=True, exist_ok=True)
    
    # Generate filename
    timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    filename = reflection_dir / f"{timestamp}.md"
    
    # Write reflection with metadata
    content = f"""# Reflection: {ken_path}

**Timestamp**: {datetime.now().isoformat()}
**Task**: {task}

---

{reflection}
"""
    
    with open(filename, 'w') as f:
        f.write(content)
    
    return filename


# ============================================================================
# Main
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Ken Bootstrap - Minimal viable wake implementation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Example:
    python ken-bootstrap.py core/kenning-parser --task "implement the parser"
    
The script will:
1. Load the kenning from kens/<path>/kenning.md
2. Walk through each frame with Claude
3. Deliver your task
4. Collect a reflection
5. Save the reflection to reflections/<path>/
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
    
    if verbose:
        print(f"Ken Wake: {args.ken_path}")
        print(f"Task: {args.task}")
        print(f"Kenning: {kenning_path}")
    
    # Parse kenning
    kenning = parse_kenning(str(kenning_path))
    
    if verbose:
        print(f"Loaded {len(kenning.frames)} frames")
    
    if not kenning.frames:
        print("Error: No frames found in kenning")
        sys.exit(1)
    
    # Create client
    client = create_client()
    
    # Walk frames
    messages = walk_frames(client, kenning, verbose)
    
    # Deliver task
    messages = deliver_task(client, messages, args.task, verbose)
    
    # Collect reflection
    reflection = collect_reflection(client, messages, verbose)
    
    # Save reflection
    reflection_file = save_reflection(args.ken_path, args.task, reflection)
    
    if verbose:
        print(f"\n{'='*60}")
        print(f"Session complete. Reflection saved to: {reflection_file}")
        print('='*60)


if __name__ == "__main__":
    main()

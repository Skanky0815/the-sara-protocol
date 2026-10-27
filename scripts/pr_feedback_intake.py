"""Validate GitHub PR feedback events without executing untrusted comment text."""
import json
import os
import subprocess
from pathlib import Path

OWNER = "Skanky0815"
AUTHOR = "Sara-Pixel42"


def eligible(event_name, event, pr_author):
    if event_name == "workflow_dispatch":
        return True
    if event_name == "pull_request":
        return pr_author == AUTHOR
    if event.get("sender", {}).get("login") != OWNER:
        return False
    if event_name == "issue_comment" and not event.get("issue", {}).get("pull_request"):
        return False
    if event_name not in {"issue_comment", "pull_request_review", "pull_request_review_comment"}:
        return False
    return pr_author == AUTHOR


def pr_number(event):
    return (event.get("pull_request") or event.get("issue") or {}).get("number")


def main():
    name = os.environ["GITHUB_EVENT_NAME"]
    event = json.loads(Path(os.environ["GITHUB_EVENT_PATH"]).read_text(encoding="utf-8"))
    if name == "workflow_dispatch":
        print("Manual smoke test: intake installed.")
        return
    number = pr_number(event)
    if not number:
        print("No PR number; ignoring.")
        return
    result = subprocess.run(
        ["gh", "api", f"repos/{os.environ['GITHUB_REPOSITORY']}/pulls/{number}", "--jq", ".user.login"],
        check=True, capture_output=True, text=True,
    )
    if not eligible(name, event, result.stdout.strip()):
        print("Not eligible; ignoring.")
        return
    print(f"Eligible feedback event: {name}, PR #{number}")
    print("Agent execution is intentionally not configured yet.")


if __name__ == "__main__":
    main()

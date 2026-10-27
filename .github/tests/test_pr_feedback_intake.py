"""Unit tests for PR feedback event eligibility."""
import unittest
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'scripts'))
from pr_feedback_intake import eligible, pr_number


class FeedbackIntakeTests(unittest.TestCase):
    def setUp(self):
        self.event = {"sender": {"login": "Skanky0815"}, "issue": {"number": 3, "pull_request": {}}}

    def test_owner_comment_on_sara_pr(self):
        self.assertTrue(eligible("issue_comment", self.event, "Sara-Pixel42"))

    def test_other_sender_ignored(self):
        self.event["sender"]["login"] = "outsider"
        self.assertFalse(eligible("issue_comment", self.event, "Sara-Pixel42"))

    def test_sara_self_comment_ignored(self):
        self.event["sender"]["login"] = "Sara-Pixel42"
        self.assertFalse(eligible("issue_comment", self.event, "Sara-Pixel42"))

    def test_regular_issue_ignored(self):
        self.event["issue"].pop("pull_request")
        self.assertFalse(eligible("issue_comment", self.event, "Sara-Pixel42"))

    def test_other_pr_author_ignored(self):
        self.assertFalse(eligible("issue_comment", self.event, "someone-else"))

    def test_review(self):
        self.assertTrue(eligible("pull_request_review", {"sender": {"login": "Skanky0815"}}, "Sara-Pixel42"))

    def test_inline_review_comment(self):
        self.assertTrue(eligible("pull_request_review_comment", {"sender": {"login": "Skanky0815"}}, "Sara-Pixel42"))

    def test_pr_smoke(self):
        self.assertTrue(eligible("pull_request", {}, "Sara-Pixel42"))

    def test_unknown_event_ignored(self):
        self.assertFalse(eligible("push", self.event, "Sara-Pixel42"))

    def test_pr_number(self):
        self.assertEqual(pr_number(self.event), 3)
        self.assertEqual(pr_number({"pull_request": {"number": 7}}), 7)


if __name__ == "__main__":
    unittest.main()

"""Independent draft readback checker; no production writer or simulated owner.

Input is captured by the real owner tests. Synthetic histories are used only to
test this checker. A passing checker unit test is never a product acceptance.
"""
from copy import deepcopy
import json


class InvalidHistory(ValueError):
    pass


def require(value, reason):
    if not value:
        raise InvalidHistory(reason)


def unique(rows, key, name):
    values = [row[key] for row in rows]
    require(len(values) == len(set(values)), f"duplicate {name}")
    return {row[key]: row for row in rows}


def validate(history, acknowledged):
    require(history["complete"] is True, "incomplete history page")
    require(history["remaining_cursor"] is None, "unconsumed history cursor")
    revisions = history["revisions"]
    require(len(revisions) > 0, "empty history")
    require(history["declared_revision_count"] == len(revisions), "omitted revision")
    by_id = unique(revisions, "revision_id", "revision")
    unique(revisions, "version", "version")
    require([r["version"] for r in revisions] == list(range(1, len(revisions) + 1)), "noncontiguous history")
    for index, row in enumerate(revisions):
        require(row["draft_id"] == history["draft_id"], "different draft")
        require(row["company_id"] == history["company_id"], "different Company")
        require(row["parent_revision_id"] == (None if index == 0 else revisions[index - 1]["revision_id"]), "broken parent")
        require(bool(row["account_id"]), "missing attribution")
        require(bool(row["schema_revision_id"]), "missing schema")
        unique(row["fields"], "address", "field address")
        for field in row["fields"]:
            require("typed_address" in field, "missing typed address")
            require(field["address"] == json.dumps(field["typed_address"], sort_keys=True, separators=(",", ":")), "noncanonical address")
    require(history["head_revision_id"] == revisions[-1]["revision_id"], "head not final revision")
    unique(acknowledged, "command_id", "acknowledged command")
    receipts = unique(history["receipts"], "command_id", "receipt command")
    require(history["declared_receipt_count"] == len(receipts), "omitted receipt")
    for ack in acknowledged:
        row = by_id.get(ack["revision_id"])
        require(row is not None, "acknowledged revision lost")
        require(row == ack["immutable_revision"], "acknowledged bytes rewritten")
        receipt = receipts.get(ack["command_id"])
        require(receipt is not None, "acknowledged receipt lost")
        require(receipt["revision_id"] == ack["revision_id"], "receipt changed revision")
        require(receipt["receipt_id"] == ack["receipt_id"], "receipt identity changed")
    return history


def extension(before, after, acknowledged, appended):
    validate(before, [a for a in acknowledged if a["revision_id"] in {r["revision_id"] for r in before["revisions"]}])
    validate(after, acknowledged)
    require(before["draft_id"] == after["draft_id"], "draft identity changed")
    require(before["company_id"] == after["company_id"], "Company changed")
    require(after["revisions"][:len(before["revisions"])] == before["revisions"], "history rewritten")
    require(len(after["revisions"]) == len(before["revisions"]) + appended, "wrong appended count")
    prior = {r["command_id"]: r for r in before["receipts"]}
    later = {r["command_id"]: r for r in after["receipts"]}
    require(all(later.get(k) == v for k, v in prior.items()), "receipt rewritten")


def selected_restore(before, after, source_id, selected):
    require(len(selected) > 0 and len(selected) == len(set(selected)), "invalid selected addresses")
    extension(before, after, [], 1)
    source = next((r for r in before["revisions"] if r["revision_id"] == source_id), None)
    require(source is not None, "source missing")
    fields = {r["address"]: r for r in before["revisions"][-1]["fields"]}
    old = {r["address"]: r for r in source["fields"]}
    require(all(k in old for k in selected), "selected field unavailable")
    expected = deepcopy(fields)
    for key in selected:
        expected[key] = old[key]
    actual = {r["address"]: r for r in after["revisions"][-1]["fields"]}
    require(actual == expected, "restore changed unselected field or omitted selected value")


def same_domain_effects(before, after):
    require(before["domain_effects"] == after["domain_effects"], "draft control caused business effect")


def single_original_effect(history, submitted_command, effect_id):
    rows = history["domain_effects"]
    unique(rows, "effect_id", "effect identity")
    require(len(rows) == 1, "missing or extra original effect")
    require(rows[0]["command_id"] == submitted_command, "wrapper replaced original command")
    require(rows[0]["effect_id"] == effect_id, "wrong effect")


if __name__ == "__main__":
    import json
    import sys
    packet = json.load(sys.stdin)
    validate(packet["history"], packet["acknowledged"])

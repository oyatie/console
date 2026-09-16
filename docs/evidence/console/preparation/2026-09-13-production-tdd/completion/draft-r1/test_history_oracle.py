import copy
import unittest
from history_oracle import InvalidHistory, validate, extension, selected_restore, same_domain_effects, single_original_effect


def row(n, fields):
    return dict(revision_id=f"r{n}", version=n, draft_id="draft", company_id="company", parent_revision_id=None if n == 1 else f"r{n-1}", account_id="account", schema_revision_id="schema1", fields=[dict(address=k, value=v) for k, v in fields.items()])


def history(rows):
    return dict(complete=True, remaining_cursor=None, declared_revision_count=len(rows), declared_receipt_count=len(rows), draft_id="draft", company_id="company", head_revision_id=rows[-1]["revision_id"], revisions=rows, receipts=[dict(command_id=f"c{i+1}", receipt_id=f"p{i+1}", revision_id=r["revision_id"]) for i,r in enumerate(rows)], domain_effects=[])


def fixture():
    h = history([row(1, {"note": "first", "amount": 3000000}), row(2, {"note": "second", "amount": 3100000})])
    a = [dict(command_id=f"c{i+1}", receipt_id=f"p{i+1}", revision_id=r["revision_id"], immutable_revision=copy.deepcopy(r)) for i,r in enumerate(h["revisions"])]
    return h,a


class HistoryOracleTests(unittest.TestCase):
    def test_accepts_actual_shape_with_every_ack(self):
        h,a=fixture();validate(h,a)

    def test_rejects_omitted_ack_even_with_self_consistent_counts(self):
        h,a=fixture();h["revisions"].pop();h["declared_revision_count"]=1;h["head_revision_id"]="r1"
        with self.assertRaisesRegex(InvalidHistory,"acknowledged revision lost"):validate(h,a)

    def test_rejects_duplicate_revision(self):
        h,a=fixture();h["revisions"][1]["revision_id"]="r1"
        with self.assertRaisesRegex(InvalidHistory,"duplicate revision"):validate(h,a)

    def test_rejects_duplicate_receipt(self):
        h,a=fixture();h["receipts"].append(h["receipts"][0]);h["declared_receipt_count"]+=1
        with self.assertRaisesRegex(InvalidHistory,"duplicate receipt"):validate(h,a)

    def test_rejects_unconsumed_page(self):
        h,a=fixture();h["remaining_cursor"]="next"
        with self.assertRaisesRegex(InvalidHistory,"unconsumed"):validate(h,a)

    def test_rejects_truncated_page(self):
        h,a=fixture();h["complete"]=False
        with self.assertRaisesRegex(InvalidHistory,"incomplete"):validate(h,a)

    def test_rejects_broken_parent(self):
        h,a=fixture();h["revisions"][1]["parent_revision_id"]="elsewhere"
        with self.assertRaisesRegex(InvalidHistory,"broken parent"):validate(h,a)

    def test_rejects_rewritten_ack(self):
        h,a=fixture();h["revisions"][0]["fields"][0]["value"]="rewritten"
        with self.assertRaisesRegex(InvalidHistory,"bytes rewritten"):validate(h,a)

    def test_rejects_cross_company_row(self):
        h,a=fixture();h["revisions"][1]["company_id"]="another"
        with self.assertRaisesRegex(InvalidHistory,"different Company"):validate(h,a)

    def test_rejects_duplicate_address(self):
        h,a=fixture();h["revisions"][0]["fields"].append(h["revisions"][0]["fields"][0])
        with self.assertRaisesRegex(InvalidHistory,"duplicate field"):validate(h,a)

    def test_accepts_selected_restore_preserving_other_field(self):
        h,a=fixture();after=history(h["revisions"]+[row(3,{"note":"first","amount":3100000})])
        selected_restore(h,after,"r1",["note"]);extension(h,after,a,1)

    def test_rejects_whole_revision_restore(self):
        h,a=fixture();after=history(h["revisions"]+[row(3,{"note":"first","amount":3000000})])
        with self.assertRaisesRegex(InvalidHistory,"unselected"):selected_restore(h,after,"r1",["note"])

    def test_rejects_draft_business_effect(self):
        h,a=fixture();after=copy.deepcopy(h);after["domain_effects"]=[dict(command_id="c",effect_id="e")]
        with self.assertRaisesRegex(InvalidHistory,"business effect"):same_domain_effects(h,after)

    def test_accepts_one_original_effect(self):
        h,a=fixture();h["domain_effects"]=[dict(command_id="original",effect_id="effect")];single_original_effect(h,"original","effect")

    def test_rejects_missing_effect(self):
        h,a=fixture()
        with self.assertRaisesRegex(InvalidHistory,"missing or extra"):single_original_effect(h,"original","effect")

    def test_rejects_wrapper_substitution(self):
        h,a=fixture();h["domain_effects"]=[dict(command_id="wrapper",effect_id="effect")]
        with self.assertRaisesRegex(InvalidHistory,"wrapper"):single_original_effect(h,"original","effect")

    def test_rejects_extra_distinct_effect(self):
        h,a=fixture();h["domain_effects"]=[dict(command_id="original",effect_id="effect"),dict(command_id="original",effect_id="extra")]
        with self.assertRaisesRegex(InvalidHistory,"missing or extra"):single_original_effect(h,"original","effect")


if __name__ == "__main__":
    unittest.main()

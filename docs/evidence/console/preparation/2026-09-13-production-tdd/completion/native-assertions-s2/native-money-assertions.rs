//! Literal existing domain golden reuse, never same-implementation calculation.
use sha2::{Digest,Sha256};
use serde_json::Value;
pub const GOLDEN_SHA256:&str="8e5411a2a891be250ddfbb16a85930449d68a964d506d3eccbbf79b9d29a53d4";
pub fn assert_native_money(durable:&Value) {
    let bytes=include_bytes!("native-money-golden.json");
    assert_eq!(hex::encode(Sha256::digest(bytes)),GOLDEN_SHA256);
    let golden:Value=serde_json::from_slice(bytes).unwrap();
    let input=&golden["input"];let expected=&golden["expected"];
    assert_eq!(durable["run"]["native_pay_date"],input["pay_date"]);
    let units=durable["units"].as_array().expect("actual resolved units");
    assert!(!units.is_empty());
    for unit in units {
        assert_eq!(unit["resolution"],"RESOLVED");
        for key in ["gross_won","taxable_monthly_won","pension_standard_monthly_income_won","monthly_standard_hours","income_tax_won","local_tax_won"] {
            assert_eq!(unit[key],input[key],"actual source semantics mismatch: {key}");
        }
        assert_eq!(unit["tax_table_edition"],"NTS_FIXTURE_ROW_V1");
    }
    let rows=durable["money"].as_array().expect("actual immutable money rows");
    assert_eq!(rows.len(),units.len());assert!(!rows.is_empty());
    for row in rows {
        for key in ["gross_won","total_deductions_won","net_won","payable"] {assert_eq!(row[key],expected[key],"literal monetary mismatch: {key}");}
        assert_eq!(row["tax_table_version"],"NTS_FIXTURE_ROW_V1");
        let deductions=row["deductions"].as_array().expect("actual six deductions");
        assert_eq!(deductions.len(),6);
        let unique:std::collections::BTreeSet<_>=deductions.iter().map(|d|d["code"].as_str().unwrap()).collect();
        assert_eq!(unique.len(),6);
        for (code,amount) in expected["deductions"].as_object().unwrap() {
            let actual=deductions.iter().find(|d|d["code"]==code.as_str()).expect("each literal component");
            assert_eq!(&actual["amount_won"],amount,"literal deduction mismatch: {code}");
        }
    }
}

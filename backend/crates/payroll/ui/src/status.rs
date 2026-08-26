//! Status chips. Never sentence-status.

#[must_use]
pub fn run_chip(status: &str) -> &'static str {
    match status {
        "STAGED" | "READY_FOR_REVIEW" => "준비",
        "BLOCKED_LEGAL_GATE" => "법적차단",
        "ATTENDANCE_CLOSED" => "근태마감",
        "CALCULATING" => "산출중",
        "CALCULATED" => "산출",
        "SUBMITTED" => "상신",
        "REJECTED" => "반려",
        "APPROVED" => "승인",
        "DISBURSEMENT_SCHEDULED" => "지급예정",
        "PAID" => "지급",
        "ISSUED" => "발행",
        "VOID" => "무효",
        _ => "상태",
    }
}

#[must_use]
pub fn exception_chip(status: &str) -> &'static str {
    match status {
        "OPEN" => "미결",
        "RESOLVED" => "해소",
        _ => "예외",
    }
}

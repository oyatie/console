//! Fresh source intervals and provenance times after all waits and decisions.
use super::*;

fn started(row: &Value, key: &str, now: OffsetDateTime) -> Result<OffsetDateTime, Error> {
    let at = time(row, key)?;
    if at > now {
        return Err(Error::Unavailable);
    }
    Ok(at)
}
fn interval(row: &Value, from: &str, until: &str, now: OffsetDateTime) -> Result<(), Error> {
    let from = started(row, from, now)?;
    let end = row.get(until).ok_or(Error::Unavailable)?;
    if !end.is_null() {
        let until = time(row, until)?;
        if until <= from {
            return Err(Error::Unavailable);
        }
        if now >= until {
            return Err(Error::Conflict);
        }
    }
    Ok(())
}

pub(super) fn valid_at(m: &Value, now: OffsetDateTime) -> Result<(), Error> {
    interval(&m["membership_revision"], "from_time", "to_time", now)?;
    for key in ["assignment_revision", "role_revision"] {
        interval(&m[key], "valid_from", "valid_until", now)?;
    }
    for clause in array(&m["registered_clauses"], 7, 7)? {
        interval(clause, "valid_from", "valid_until", now)?;
    }
    for key in [
        "account",
        "company",
        "group",
        "company_actor",
        "assignment",
        "role",
        "assignment_revision",
        "role_revision",
        "birth_request",
    ] {
        let row = &m[key];
        let created = started(row, "created_at", now)?;
        if row.get("updated_at").is_some() && started(row, "updated_at", now)? < created {
            return Err(Error::Unavailable);
        }
    }
    let committed = started(&m["birth_receipt"], "committed_at", now)?;
    if started(&m["birth_effect"], "started_at", now)? != committed
        || started(&m["birth_request"], "terminal_at", now)? != committed
        || started(&m["company_actor"], "created_at", now)? != committed
    {
        return Err(Error::Unavailable);
    }
    let family = &m["family"]["material"];
    if started(family, "auth_time", now)? > started(family, "created_at", now)? {
        return Err(Error::AuthenticationInvalid);
    }
    for row in array(&m["birth_events"], 2, 2)? {
        started(row, "occurred_at", now)?;
    }
    for row in array(&m["registration"], 1, 8)? {
        started(row, "accepted_at", now)?;
    }
    started(&m["account"]["security"], "updated_at", now)?;
    for row in array(&m["catalog"]["installs"], 1, 3)? {
        started(row, "installed_at", now)?;
    }
    for entry in array(&m["catalog"]["initial_source_attribution"], 3, 3)? {
        let row = entry
            .get("row")
            .filter(|row| row.is_object())
            .ok_or(Error::Unavailable)?;
        for key in ["created_at", "updated_at", "installed_at"] {
            if row.get(key).is_some() {
                started(row, key, now)?;
            }
        }
    }
    Ok(())
}

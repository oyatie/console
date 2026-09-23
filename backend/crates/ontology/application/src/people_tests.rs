use crate::people::{
    DirectoryInputError, DirectoryInputField, DirectoryInputProblem, DirectoryRegistrationInput,
};

#[test]
fn directory_input_preserves_korean_and_distinct_unicode_identities() {
    let input =
        DirectoryRegistrationInput::new("\u{2003}김하늘 <연구 & 운영>\u{a0}", "  UI-사람-001  ")
            .unwrap();
    assert_eq!(input.legal_name(), "김하늘 <연구 & 운영>");
    assert_eq!(input.employee_number(), "UI-사람-001");
    for (left, right) in [("é", "e\u{301}"), ("Ａ", "A"), ("ab", "AB"), ("a b", "ab")] {
        let a = DirectoryRegistrationInput::new(left, left).unwrap();
        let b = DirectoryRegistrationInput::new(right, right).unwrap();
        assert_ne!(a.legal_name(), b.legal_name());
        assert_ne!(a.employee_number(), b.employee_number());
    }
}

#[test]
fn directory_input_requires_both_fields_and_counts_unicode_scalars() {
    use DirectoryInputField::{EmployeeNumber, LegalName};
    use DirectoryInputProblem::{Required, TooLong};
    for raw in ["", " ", "\u{2003}\u{a0}"] {
        assert_eq!(
            DirectoryRegistrationInput::new(raw, "1").unwrap_err(),
            DirectoryInputError {
                field: LegalName,
                problem: Required
            }
        );
        assert_eq!(
            DirectoryRegistrationInput::new("이름", raw).unwrap_err(),
            DirectoryInputError {
                field: EmployeeNumber,
                problem: Required
            }
        );
    }
    for character in ['가', '🧑'] {
        assert!(
            DirectoryRegistrationInput::new(
                &character.to_string().repeat(200),
                &character.to_string().repeat(64)
            )
            .is_ok()
        );
        assert_eq!(
            DirectoryRegistrationInput::new(&character.to_string().repeat(201), "1").unwrap_err(),
            DirectoryInputError {
                field: LegalName,
                problem: TooLong
            }
        );
        assert_eq!(
            DirectoryRegistrationInput::new("이름", &character.to_string().repeat(65)).unwrap_err(),
            DirectoryInputError {
                field: EmployeeNumber,
                problem: TooLong
            }
        );
    }
}

#[test]
fn directory_input_rejects_controls_even_when_trimming_would_hide_them() {
    use DirectoryInputField::{EmployeeNumber, LegalName};
    use DirectoryInputProblem::ControlCharacter;
    for control in ['\0', '\n', '\r', '\t', '\u{7f}', '\u{85}', '\u{9f}'] {
        for raw in [
            format!("{control}이름"),
            format!("이{control}름"),
            format!("이름{control}"),
        ] {
            assert_eq!(
                DirectoryRegistrationInput::new(&raw, "1").unwrap_err(),
                DirectoryInputError {
                    field: LegalName,
                    problem: ControlCharacter
                }
            );
            assert_eq!(
                DirectoryRegistrationInput::new("이름", &raw).unwrap_err(),
                DirectoryInputError {
                    field: EmployeeNumber,
                    problem: ControlCharacter
                }
            );
        }
    }
}

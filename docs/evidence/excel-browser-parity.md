# Excel-to-browser parity verification assets

- Source workbook: `/Users/jasonlee/Downloads/Untitled spreadsheet.xlsx`
- SHA-256: `d844d5309dd8a0c48d3a2de5175a29165e5a950c485fd6707bcff259f9e3eda1`
- Row rule: browser employee rows are workbook rows below the header with a nonblank `성명`/name cell. Blank-name template/staging rows are preserved in raw import evidence but are not displayed as employees.

## Expected company counts and spot-check names

| Sheet | Browser filter value | Expected browser rows | Source template rows | First name | Last name |
| --- | --- | ---: | ---: | --- | --- |
| `(주)예시나` | `(주)예시나` | 13 | 95 | 홍길동 | 김철수 |
| `(주)코스` | `(주)코스` | 96 | 96 | 이영희 | 박민수 |
| `(주)예시마` | `(주)예시마` | 139 | 139 | 최지우 | 정하늘 |
| `(주)예시차` | `(주)예시차` | 4 | 68 | 강바다 | 조은별 |
| `(주)예시자로지스` | `(주)예시자로지스` | 1 | 68 | 윤가람 | 장나래 |
| `(주)씨앤엘` | `(주)씨앤엘` | 43 | 68 | 임다온 | 한라온 |
| `(주)예시자HR` | `(주)예시자HR` | 22 | 62 | 오새봄 | 서여름 |
| `예시사` | `예시사` | 11 | 11 | 신가을 | 권겨울 |

## CP949 CSV encoding evidence

- Fixture: `e2e/.artifacts/org-cp949-evidence.csv`
- Written as `cp949`, decoded with browser-compatible label `euc-kr`.
- Headers after decode: `회사명, 부서명, 이름`
- Replacement characters: `0`
- SHA-256: `ff3ea83090befe339f88f2528144006a9ad98590c3a98cd25fa2383b8b6e5f2b`

## Commands

```bash
python3 scripts/derive_excel_browser_parity.py --output e2e/.artifacts/excel-parity-expected.json --cp949-csv e2e/.artifacts/org-cp949-evidence.csv
npx playwright test e2e/specs/admin-20-excel-browser-parity.spec.ts
E2E_EXCEL_PARITY=1 e2e/run.sh e2e/specs/admin-20-excel-browser-parity.spec.ts
```

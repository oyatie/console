> **SUPERSEDED — 2026-07-28 pivot.** This is the sync record of Claude Design project `9c7c313a`
> ("Oyatie Console"), which is **no longer the design authority** (current: project `198fcee4`
> "기본", shell only) — there will be no further sync passes, and the wireframes it describes are
> historical. The repo line below has been corrected for the `maintenance` → `console` rename; the
> `docs/` paths in the tables were verified to still exist on 2026-07-28, but their contents were
> read before the pivot. Truth set: [`docs/PIVOT-2026-07-28.md`](../../PIVOT-2026-07-28.md).

repo: jason931225/console
branch: main
path: docs/

## Last sync
date: 2026-07-25T05:06:00Z
scope: 문서 대조만(코드 미이관) — 설계·구현 상태를 와이어프레임에 반영

### Updated in this project
- 「Oyatie 구조 · 관계 맵 (와이어프레임)」 1i 신설: 실제 구현 스택(Rust 모듈러 모놀리스·passkey·RLS·SeaweedFS WORM·OCI/Talos·Argo)과 엔터프라이즈 성숙도 정직 판정(관측성 미배포·단일 노드 HA 상한·급여 법무 게이트).
- 1g 법규 배선도 확장: 위치정보법(동의·주기·보존·분리 저장)·passkey step-up(서명 등가 액션)·초기 로그인 동의·임금대장 산정 입력 보존.
- 그룹 내 전적 = 퇴직 에피소드 + 신규 입사 에피소드로 정정(단순 전보 표기 오류) · 고용 에피소드를 데이터 모델에 반영.
- 로드맵 시퀀스·MES 벤치마크(SAP DM·Opcenter·Plex·Tulip·Dynamics) 조율 필요로 기록.

## Screen map
| 우리 산출물 | 참조한 리포 파일 |
|---|---|
| 와이어프레임 1i (구현 현실·성숙도) | docs/ENTERPRISE-READINESS.md · docs/PLATFORM-ROADMAP.md · docs/decisions/ADR-0001~0026 |
| 와이어프레임 1g (법규 배선도) | docs/specs/korean-legal-boundaries.md |
| 와이어프레임 1c (개체 소유·이관) | docs/specs/korean-legal-boundaries.md(전적·에피소드) · ADR-0002/0003/0014 |
| 와이어프레임 1h (커버리지·갭) | docs/PLATFORM-ROADMAP.md(시퀀스) · docs/specs/mes.md(벤치마크 지정) |

## 다음 대조 후보 (미독)
- docs/benchmarks/enterprise-parity-matrix.md (44k) · docs/program/ontology-coverage-matrix.md · docs/specs/no-code-operational-logic.md · docs/specs/cedar-pbac-authorization.md · docs/web-console-overhaul-spec.md

import { orgStrings as text } from "../../i18n/org";
import type {
  BranchSummary,
  HrOrgChartCompany,
  HrOrgChartResponse,
  HrOrgChartUnit,
  OrgEntitySummary,
  OrgProposalOp,
  RegionSummary,
} from "./orgApi";

export interface OrgTreeSite {
  branch: BranchSummary;
  regionName: string;
  /** Sandbox proposal projections (design §3.9.0 — 임시, 개편 결재로 확정). */
  pendingNew?: boolean;
  pendingOff?: boolean;
}

export interface OrgTreeHead {
  id: string;
  name: string;
  title: string;
}

export interface OrgTreeColumn {
  /** Company display name (hr org-chart TEXT grouping, or entity name). */
  company: string;
  /** Group entity matched by name; absent = unmapped chart company. */
  entity?: OrgEntitySummary;
  /** hr org-chart company node; absent = entity without employee rows. */
  chart?: HrOrgChartCompany;
  /** Identity regions/branches attach only to the viewer's own 법인 column. */
  sites: OrgTreeSite[];
  units: HrOrgChartUnit[];
  active: number;
  total: number;
}

/**
 * Merge the three real backend sources into design columns. Mapping between
 * the hr `company` TEXT grouping and group entities is name-based (scout
 * digest): unmapped chart companies stay as explicit columns, never dropped;
 * entities without employees render as empty columns.
 */
export function buildOrgColumns(input: {
  chart: HrOrgChartResponse;
  entities: OrgEntitySummary[];
  regions: RegionSummary[];
  branches: BranchSummary[];
  viewerCompany: string | null;
}): OrgTreeColumn[] {
  const regionName = new Map(input.regions.map((region) => [region.id, region.name]));
  const sites: OrgTreeSite[] = input.branches.map((branch) => ({
    branch,
    regionName: regionName.get(branch.region_id) ?? "",
  }));
  const byCompany = new Map(input.chart.companies.map((company) => [company.company, company]));
  const claimed = new Set<string>();

  const columns: OrgTreeColumn[] = input.entities.map((entity) => {
    const chart = byCompany.get(entity.name);
    if (chart) claimed.add(chart.company);
    return {
      company: entity.name,
      entity,
      chart,
      sites: [],
      units: chart?.units ?? [],
      active: chart?.active ?? 0,
      total: chart?.total ?? 0,
    };
  });

  for (const company of input.chart.companies) {
    if (claimed.has(company.company)) continue;
    columns.push({
      company: company.company,
      chart: company,
      sites: [],
      units: company.units,
      active: company.active,
      total: company.total,
    });
  }

  if (sites.length > 0) {
    const owner = input.viewerCompany
      ? columns.find((column) => column.company === input.viewerCompany)
      : undefined;
    if (owner) {
      owner.sites = sites;
    } else {
      columns.push({
        company: text.unmappedColumn,
        sites,
        units: [],
        active: 0,
        total: 0,
      });
    }
  }

  return columns;
}

/**
 * 책임자 derivation from the org-chart object — leader-titled position lookup,
 * no hardcoded names (design §: "조직도 개체에서 리더 직급자 자동 조회").
 */
export function deriveUnitHead(unit: HrOrgChartUnit): OrgTreeHead | undefined {
  for (const position of unit.positions) {
    if (!position.title.endsWith(text.headTitleSuffix)) continue;
    const employee = position.employees.at(0);
    if (employee) return { id: employee.id, name: employee.name, title: position.title };
  }
  return undefined;
}

/** 재직 headcount for the group root card — sum of company actives (derived). */
export function totalActive(columns: OrgTreeColumn[]): number {
  return columns.reduce((sum, column) => sum + column.active, 0);
}

/**
 * Project the sandbox proposal onto the loaded tree so inline edits render as
 * the (banner-labelled) 임시 state they are. Pure — the server proposal replay
 * at effectuate is the authority.
 */
export function applyPendingOps(
  columns: OrgTreeColumn[],
  ops: OrgProposalOp[],
  siteOwner: string | null,
): OrgTreeColumn[] {
  if (ops.length === 0) return columns;
  return columns.map((column) => {
    let sites = column.sites.map((site) => ({ ...site }));
    let units = column.units.map((unit) => ({ ...unit }));
    for (const op of ops) {
      switch (op.op) {
        case "RENAME_BRANCH":
          sites = sites.map((site) =>
            site.branch.id === op.branchId && op.name
              ? { ...site, branch: { ...site.branch, name: op.name } }
              : site,
          );
          break;
        case "DEACTIVATE_BRANCH":
          sites = sites.map((site) =>
            site.branch.id === op.branchId ? { ...site, pendingOff: true } : site,
          );
          break;
        case "CREATE_BRANCH":
          if (column.company === siteOwner) {
            sites = [...sites, {
              branch: {
                id: `pending:${op.name}`,
                region_id: op.regionId,
                name: op.name,
                deactivated_at: null,
                created_at: "",
              },
              regionName: "",
              pendingNew: true,
            }];
          }
          break;
        case "REASSIGN_ORG_UNIT":
          if (op.scope.company === column.company) {
            const target = units.find((unit) => unit.name === op.toOrgUnit);
            const source = units.find((unit) => unit.name === op.fromOrgUnit);
            if (source && target && source !== target) {
              target.total += source.total;
              // Merge by title so the projected card keeps one row per 직급.
              const merged = target.positions.map((position) => ({ ...position }));
              for (const position of source.positions) {
                const existing = merged.find((candidate) => candidate.title === position.title);
                if (existing) {
                  existing.total += position.total;
                  existing.employees = [...existing.employees, ...position.employees];
                } else {
                  merged.push({ ...position });
                }
              }
              target.positions = merged;
              units = units.filter((unit) => unit !== source);
            } else if (source) {
              source.name = op.toOrgUnit;
            }
          }
          break;
        // Regions and registry sites are not rows of this tree — no projection.
        case "CREATE_REGION":
        case "RENAME_REGION":
        case "DEACTIVATE_REGION":
        case "CREATE_SITE":
        case "UPDATE_SITE":
          break;
      }
    }
    return { ...column, sites, units };
  });
}

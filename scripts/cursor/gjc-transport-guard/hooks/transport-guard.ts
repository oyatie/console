/**
 * Constrained GJC plugin hook: intercepts bash/edit/write/ast_edit/apply_patch
 * tool_call events and applies the shared hub-aware transport policy.
 *
 * The factory may only register the declared event. Host mutation APIs are denied.
 *
 * cwd comes from hook context (session cwd). bash may also set input.cwd; the
 * policy resolves that override. Empty cwd cannot satisfy a hub product-write claim.
 */
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const GUARDED = new Set(["bash", "edit", "write", "ast_edit", "apply_patch"]);

type PolicyMod = {
	evaluateToolCall: (call: {
		toolName: string;
		input?: Record<string, unknown>;
		cwd?: string;
	}) => { decision: "ALLOW" | "DENY"; reason: string; block: boolean };
};

async function loadPolicy(): Promise<PolicyMod> {
	const hookDir = dirname(fileURLToPath(import.meta.url));
	const policyUrl = pathToFileURL(join(hookDir, "..", "policy.mjs")).href;
	return (await import(policyUrl)) as PolicyMod;
}

export default async function registerTransportGuard(api: {
	on: (event: string, handler: (...args: unknown[]) => unknown) => void;
}) {
	const policy = await loadPolicy();
	api.on("tool_call", (event: unknown, ctx: unknown) => {
		const ev = event && typeof event === "object" ? (event as Record<string, unknown>) : {};
		const context = ctx && typeof ctx === "object" ? (ctx as Record<string, unknown>) : {};
		const toolName = String(ev.toolName ?? ev.tool_name ?? ev.name ?? "");
		if (!GUARDED.has(toolName)) return {};
		const input = ev.input && typeof ev.input === "object" ? (ev.input as Record<string, unknown>) : {};
		const cwd = typeof context.cwd === "string" ? context.cwd : "";
		const verdict = policy.evaluateToolCall({ toolName, input, cwd });
		if (verdict.decision === "DENY" || verdict.block) {
			return { block: true, reason: verdict.reason };
		}
		return {};
	});
}

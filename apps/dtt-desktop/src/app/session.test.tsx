import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { SaveAndEmpirePage } from "@/features/saves/SaveAndEmpirePage";
import { initializeI18n } from "@/i18n";
import { AppShell } from "./AppShell";
import { INITIAL_SESSION_STATE, canStartGeneration, generationBlocker, hasValidEnvironment, sessionReducer, type SessionState } from "./session";
import { createEnvironment, createInspection, createReadySession, createSave } from "./test-fixtures";

vi.mock("@/i18n/controller", () => ({
  useLocale: () => ({ preference: "en" }),
  currentLocale: () => "en",
}));

const environment = createEnvironment();
const inspection = createInspection();
let ready: SessionState;

beforeEach(() => {
  ready = createReadySession();
});

beforeAll(async () => { await initializeI18n("en"); });

function shell(state: SessionState): string {
  return renderToStaticMarkup(<AppShell
    state={state} onNavigateStep={vi.fn()} onStartGeneration={vi.fn()} onCancelGeneration={vi.fn()}
    onOpenOutputDirectory={vi.fn()} onChangeSave={vi.fn()} onChangeLocale={vi.fn()} onChangeTheme={vi.fn()} currentTheme="system"
  >{null}</AppShell>);
}

function buttonIsDisabled(html: string, label: string): boolean {
  const button = html.match(/<button\b[^>]*>[\s\S]*?<\/button>/g)?.find(value => value.includes(label));
  expect(button).toBeDefined();
  return /\sdisabled(?:=|[ >])/.test(button!);
}

describe("generation readiness", () => {
  it("reflects generation readiness in the start button", () => {
    expect(buttonIsDisabled(shell(ready), "Start Generation")).toBe(false);
    expect(buttonIsDisabled(shell({ ...ready, environment: null }), "Start Generation")).toBe(true);
  });

  it.each<{ reason: string; patch: Partial<SessionState> }>([
    { reason: "missing environment", patch: { environment: null } },
    { reason: "invalid environment", patch: { environmentError: { code: "INVALID_PATH" } } },
    { reason: "pending inspection", patch: { inspectionStatus: "inspecting" } },
    { reason: "failed inspection", patch: { inspectionStatus: "failed" } },
    { reason: "missing inspection", patch: { inspection: null } },
    { reason: "binary save", patch: { selectedSave: { ...createSave(), state: "binary" } } },
  ])("disables generation for $reason", ({ patch }) => {
    const state = { ...ready, ...patch };
    expect(canStartGeneration(state)).toBe(false);
  });

  it("requires an explicit multiplayer country selection", () => {
    const state = { ...ready, inspection: { ...inspection, playerCountries: [{ countryId: 1 }, { countryId: 2 }] }, selectedCountryId: null };
    expect(generationBlocker(state)).toBe("empire");
    expect(canStartGeneration({ ...state, selectedCountryId: 2 })).toBe(true);
  });

  it("disables the settings next button without a valid environment", () => {
    const state = { ...INITIAL_SESSION_STATE, bootstrap: { status: "ready" as const, data: { environment, outputDirectory: "output" } } };
    expect(buttonIsDisabled(shell(state), "Next")).toBe(true);
    expect(buttonIsDisabled(shell({ ...state, environment }), "Next")).toBe(false);
  });

  it("invalidates the environment immediately when paths change or resolution starts", () => {
    for (const state of [
      sessionReducer(ready, { type: "SET_OVERRIDE", key: "gameRoot", value: "other" }),
      sessionReducer(ready, { type: "RESOLVE_ENV_START", seq: 1 }),
    ]) {
      expect(hasValidEnvironment(state)).toBe(false);
      expect(canStartGeneration(state)).toBe(false);
    }
  });

  it("does not restore bootstrap paths over pending overrides or newer resolution", () => {
    const bootstrap = { type: "BOOTSTRAP_SUCCESS" as const, data: { environment, outputDirectory: "output" } };
    const pending = sessionReducer(ready, { type: "RESOLVE_ENV_START", seq: 2 });
    const restored = { ...INITIAL_SESSION_STATE, overrides: { ...INITIAL_SESSION_STATE.overrides, gameRoot: "custom" } };
    expect(sessionReducer(pending, bootstrap).environment).toBeNull();
    expect(sessionReducer(restored, bootstrap).environment).toBeNull();
    const stale = sessionReducer(pending, { type: "RESOLVE_ENV_SUCCESS", environment, seq: 1 });
    expect(stale.environment).toBeNull();
    expect(sessionReducer(pending, { type: "RESOLVE_ENV_SUCCESS", environment, seq: 2 }).environment).toEqual(environment);
  });
});

it("shows the country selector before a multiplayer snapshot exists", () => {
  const state: SessionState = { ...ready, selectedCountryId: null, inspection: { playerCountries: [{ countryId: 1 }, { countryId: 2 }] } };
  const html = renderToStaticMarkup(<SaveAndEmpirePage
    state={state} dispatch={vi.fn()} onScanLibrary={vi.fn()} onSelectSave={vi.fn()} onInspectWithCountry={vi.fn()}
  />);
  expect(html).toContain('aria-haspopup="menu"');
  expect(html).toContain("Choose a player country to inspect its empire.");
});

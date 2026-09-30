import { beforeAll, describe, expect, it, vi } from "vitest";
import { renderToStaticMarkup } from "react-dom/server";
import { SaveAndEmpirePage } from "@/features/saves/SaveAndEmpirePage";
import { initializeI18n } from "@/i18n";
import type { EnvironmentDto, SaveIndexDto, InspectedSaveDto } from "@/ipc/bindings";
import { AppShell } from "./AppShell";
import { INITIAL_SESSION_STATE, canStartGeneration, generationBlocker, hasValidEnvironment, sessionReducer, type SessionState } from "./session";

vi.mock("@/i18n/controller", () => ({
  useLocale: () => ({ preference: "en" }),
  currentLocale: () => "en",
}));

const environment: EnvironmentDto = { gameRoot: "game", documentsDir: "documents", launcherDb: "launcher.sqlite", steamLibraries: [] };
const save: SaveIndexDto = { path: "example.sav", fileName: "example.sav", modifiedAtMillis: 0, fileSize: 1, state: "text" };
const inspection: InspectedSaveDto = {
  playerCountries: [{ countryId: 1 }],
  snapshot: { ethics: [], government: { civics: [] }, ascensionPerks: [], traditions: [], founderSpecies: { traits: [] }, countryType: "default" },
};
const ready: SessionState = {
  ...INITIAL_SESSION_STATE, step: "saves", environment, selectedSave: save, selectedSavePath: save.path,
  inspection, inspectionStatus: "ready", selectedCountryId: 1,
};

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
  it("enables generation only for a validated environment and inspected empire", () => {
    expect(canStartGeneration(ready)).toBe(true);
    expect(buttonIsDisabled(shell(ready), "Start Generation")).toBe(false);
    for (const patch of [
      { environment: null },
      { environmentError: { code: "INVALID_PATH" as const } },
      { inspectionStatus: "inspecting" as const },
      { inspectionStatus: "failed" as const },
      { inspection: null },
      { selectedSave: { ...save, state: "binary" as const } },
    ]) {
      const state = { ...ready, ...patch };
      expect(canStartGeneration(state)).toBe(false);
      expect(buttonIsDisabled(shell(state), "Start Generation")).toBe(true);
    }
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

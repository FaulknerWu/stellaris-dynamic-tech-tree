import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { Children, isValidElement, type ComponentProps, type ComponentType, type ReactElement, type ReactNode } from "react";
import type { BootstrapDataDto, EnvironmentDto, InspectedSaveDto, SaveLibraryDto } from "@/ipc/bindings";
import { SaveAndEmpirePage } from "@/features/saves/SaveAndEmpirePage";
import { EnvironmentAndSettingsPage } from "@/features/settings/EnvironmentAndSettingsPage";
import { App } from "./App";
import { AppShell } from "./AppShell";
import { INITIAL_SESSION_STATE, sessionReducer, type SessionAction, type SessionState } from "./session";
import { DEFAULT_PERSISTED_DATA } from "./store";
import { createEnvironment, createReadySession, createSave as save } from "./test-fixtures";

const harness = vi.hoisted(() => ({
  state: null as SessionState | null,
  refs: [] as Array<{ current: unknown }>,
  refIndex: 0,
  effects: [] as Array<() => void | (() => void)>,
  cleanups: [] as Array<() => void>,
  dispatch: vi.fn<(action: SessionAction) => void>(),
  scan: vi.fn(), inspect: vi.fn(), persist: vi.fn(), resolve: vi.fn(), bootstrap: vi.fn(), generate: vi.fn(),
}));

vi.mock("react", async original => ({
  ...await original<typeof import("react")>(),
  useEffect: (effect: () => void | (() => void)) => { harness.effects.push(effect); },
  useReducer: () => [harness.state, harness.dispatch],
  useRef: (current: unknown) => {
    const index = harness.refIndex++;
    return harness.refs[index] ?? (harness.refs[index] = { current });
  },
  useState: (initial: unknown) => [initial, vi.fn()],
  useCallback: (callback: unknown) => callback,
}));
vi.mock("react-i18next", () => ({ useTranslation: () => ({ t: () => "" }) }));
vi.mock("@/i18n/controller", () => ({
  useLocale: () => ({ saveFailed: false }), currentLocale: () => "en",
  listenForSystemLocale: vi.fn(), setLocalePreference: vi.fn(),
}));
vi.mock("./theme", () => ({ applyTheme: vi.fn() }));
vi.mock("./store", async original => ({
  ...await original<typeof import("./store")>(), savePersistedData: harness.persist,
}));
vi.mock("@/ipc/commands", () => ({
  scanSaveLibrary: harness.scan, inspectSave: harness.inspect, resolveEnvironment: harness.resolve,
  getBootstrapData: harness.bootstrap, runGeneration: harness.generate, cancelGeneration: vi.fn(), openOutputDirectory: vi.fn(),
}));

const environment = createEnvironment();
const inspection: InspectedSaveDto = { playerCountries: [{ countryId: 1 }] };
const library = (path: string): SaveLibraryDto => ({ diagnostics: [], accounts: [{ campaigns: [{ key: path, saves: [save(path)] }] }] });

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}

// Exercise callback races directly; mount effects only for bootstrap scenarios.
function render({ mountEffects = false } = {}) {
  harness.refIndex = 0;
  harness.effects = [];
  const tree = App({ persisted: structuredClone(DEFAULT_PERSISTED_DATA), initialSaveFailed: false }) as ReactElement<ComponentProps<typeof AppShell>>;
  if (mountEffects) {
    for (const effect of harness.effects) {
      const cleanup = effect();
      if (cleanup) harness.cleanups.push(cleanup);
    }
  }
  return tree;
}

function pageProps<P>(tree: ReactElement<{ children: ReactNode }>, component: ComponentType<P>): P {
  const child = Children.toArray(tree.props.children).find(value => isValidElement(value) && value.type === component);
  expect(child).toBeDefined();
  return (child as ReactElement<P>).props;
}

beforeEach(() => {
  vi.resetAllMocks();
  harness.refs = [];
  harness.state = { ...structuredClone(INITIAL_SESSION_STATE), step: "saves", environment: createEnvironment() };
  harness.dispatch.mockImplementation(action => { harness.state = sessionReducer(harness.state!, action); });
  harness.inspect.mockResolvedValue(inspection);
  harness.persist.mockResolvedValue(undefined);
  harness.scan.mockResolvedValue({ diagnostics: [], accounts: [] });
});
afterEach(() => {
  for (const cleanup of harness.cleanups.splice(0)) cleanup();
  vi.clearAllTimers();
  vi.useRealTimers();
});

it("ignores every side effect of a superseded library scan", async () => {
  const local = deferred<SaveLibraryDto>();
  const cloud = deferred<SaveLibraryDto>();
  harness.scan.mockReturnValueOnce(local.promise).mockReturnValueOnce(cloud.promise);
  const page = pageProps(render(), SaveAndEmpirePage);
  const oldScan = page.onScanLibrary("local");
  harness.dispatch({ type: "SET_SAVE_SOURCE", source: "steam_cloud" });
  const currentScan = page.onScanLibrary("steam_cloud");
  cloud.resolve(library("cloud.sav"));
  await currentScan;
  local.resolve(library("local.sav"));
  await oldScan;
  expect(harness.state?.library).toEqual(library("cloud.sav"));
  expect(harness.state?.selectedSavePath).toBe("cloud.sav");
  expect(harness.inspect).toHaveBeenCalledExactlyOnceWith({ saveFile: "cloud.sav" });
  expect(harness.persist).toHaveBeenCalledExactlyOnceWith({ saveSource: "steam_cloud" });
});

it("does not replace a manual selection made while scanning", async () => {
  const pending = deferred<SaveLibraryDto>();
  harness.scan.mockReturnValueOnce(pending.promise);
  const page = pageProps(render(), SaveAndEmpirePage);
  const scanning = page.onScanLibrary();
  page.onSelectSave(save("manual.sav"));
  pending.resolve(library("automatic.sav"));
  await scanning;
  expect(harness.state?.selectedSavePath).toBe("manual.sav");
  expect(harness.inspect).toHaveBeenCalledExactlyOnceWith({ saveFile: "manual.sav" });
});

it("ignores inspection completion after resetting the selection", async () => {
  const pending = deferred<InspectedSaveDto>();
  harness.inspect.mockReturnValueOnce(pending.promise);
  const tree = render();
  pageProps(tree, SaveAndEmpirePage).onSelectSave(save("old.sav"));
  tree.props.onChangeSave();
  pending.resolve(inspection);
  await pending.promise;
  expect(harness.state?.selectedSavePath).toBeNull();
  expect(harness.state?.inspection).toBeNull();
  expect(harness.state?.inspectionStatus).toBe("idle");
});

it("ignores a scan failure after a newer scan succeeds", async () => {
  const pending = deferred<SaveLibraryDto>();
  harness.scan.mockReturnValueOnce(pending.promise);
  const page = pageProps(render(), SaveAndEmpirePage);
  const oldScan = page.onScanLibrary();
  await page.onScanLibrary();
  pending.reject({ code: "INTERNAL" });
  await oldScan;
  expect(harness.state?.libraryStatus).toBe("ready");
  expect(harness.state?.libraryError).toBeNull();
});

it("invalidates environment responses as soon as a newer request is scheduled", async () => {
  vi.useFakeTimers();
  harness.state = { ...harness.state!, step: "settings" };
  const old = deferred<EnvironmentDto>();
  const current = deferred<EnvironmentDto>();
  harness.resolve.mockReturnValueOnce(old.promise).mockReturnValueOnce(current.promise);
  const page = pageProps(render(), EnvironmentAndSettingsPage);
  page.onResolveEnvironment({ ...INITIAL_SESSION_STATE.overrides, gameRoot: "old" });
  await vi.advanceTimersByTimeAsync(300);
  page.onResolveEnvironment({ ...INITIAL_SESSION_STATE.overrides, gameRoot: "new" });
  old.resolve({ ...environment, gameRoot: "old" });
  await old.promise;
  expect(harness.state?.environment).toBeNull();
  expect(harness.persist).not.toHaveBeenCalled();
  expect(harness.scan).not.toHaveBeenCalled();
  await vi.advanceTimersByTimeAsync(300);
  current.resolve({ ...environment, gameRoot: "new" });
  await current.promise;
  expect(harness.state?.environment?.gameRoot).toBe("new");
  expect(harness.persist).toHaveBeenCalledWith({ overrides: { ...INITIAL_SESSION_STATE.overrides, gameRoot: "new" } });
});

it("does not let late bootstrap data restart a scan after a path change", async () => {
  vi.useFakeTimers();
  harness.state = { ...INITIAL_SESSION_STATE };
  const pending = deferred<BootstrapDataDto>();
  harness.bootstrap.mockReturnValueOnce(pending.promise);
  const page = pageProps(render({ mountEffects: true }), EnvironmentAndSettingsPage);
  page.onResolveEnvironment({ ...INITIAL_SESSION_STATE.overrides, gameRoot: "new" });
  pending.resolve({ environment, outputDirectory: "output" });
  await pending.promise;
  expect(harness.state?.environment).toBeNull();
  expect(harness.scan).not.toHaveBeenCalled();
});

it("rejects starting generation while environment validation is incomplete", async () => {
  harness.state = { ...createReadySession(), environment: null };
  await render().props.onStartGeneration();
  expect(harness.generate).not.toHaveBeenCalled();
});

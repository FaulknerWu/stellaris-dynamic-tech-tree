import type { EnvironmentDto, InspectedSaveDto, SaveIndexDto } from "@/ipc/bindings";
import { INITIAL_SESSION_STATE, type SessionState } from "./session";

export function createEnvironment(): EnvironmentDto {
  return { gameRoot: "game", documentsDir: "documents", launcherDb: "launcher.sqlite", steamLibraries: [] };
}

export function createSave(path = "example.sav"): SaveIndexDto {
  return { path, fileName: path, modifiedAtMillis: 0, fileSize: 1, state: "text" };
}

export function createInspection(): InspectedSaveDto {
  return {
    playerCountries: [{ countryId: 1 }],
    snapshot: {
      ethics: [], government: { civics: [] }, ascensionPerks: [], traditions: [],
      founderSpecies: { traits: [] }, countryType: "default",
    },
  };
}

export function createReadySession(): SessionState {
  const save = createSave();
  return {
    ...structuredClone(INITIAL_SESSION_STATE),
    step: "saves", environment: createEnvironment(), selectedSave: save, selectedSavePath: save.path,
    inspection: createInspection(), inspectionStatus: "ready", selectedCountryId: 1,
  };
}

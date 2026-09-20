import type { GenerationDiagnosticItemDto } from "./GenerationDiagnosticItemDto";

export type GenerationDiagnosticsDto = {
  loadOrder: Array<GenerationDiagnosticItemDto>;
  unknownConditions: Array<GenerationDiagnosticItemDto>;
  deferredConditions: Array<GenerationDiagnosticItemDto>;
  gameData: Array<GenerationDiagnosticItemDto>;
  unhandledDefinitions: Array<GenerationDiagnosticItemDto>;
  localisation: Array<GenerationDiagnosticItemDto>;
  cycles: Array<GenerationDiagnosticItemDto>;
  writeFailures: Array<GenerationDiagnosticItemDto>;
};

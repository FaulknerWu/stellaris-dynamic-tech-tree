import type { GenerationDiagnosticsDto } from "./GenerationDiagnosticsDto";
import type { GenerationStatusDto } from "./GenerationStatusDto";

export type GenerationResultDto = {
  status: GenerationStatusDto;
  sourceCount: number;
  technologyCount: number;
  eligibleCount: number;
  swapMatched: number;
  swapNoMatch: number;
  swapUncertain: number;
  written: Array<string>;
  removed: Array<string>;
  failed: Array<string>;
  reportPath?: string | undefined;
  diagnostics: GenerationDiagnosticsDto;
};

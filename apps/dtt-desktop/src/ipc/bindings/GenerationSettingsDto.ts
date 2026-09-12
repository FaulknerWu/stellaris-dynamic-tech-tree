import type { SupportedLanguageDto } from "./SupportedLanguageDto";
import type { SwapUnknownStrategyDto } from "./SwapUnknownStrategyDto";
import type { UnknownStrategyDto } from "./UnknownStrategyDto";

export type GenerationSettingsDto = {
  stellarisRoot: string;
  launcherDb: string;
  languages: Array<SupportedLanguageDto>;
  unknownStrategy: UnknownStrategyDto;
  swapUnknownStrategy: SwapUnknownStrategyDto;
};

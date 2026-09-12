import type { GenerationSettingsDto } from "./GenerationSettingsDto";

export type GenerationRequestDto = {
  saveFile: string;
  countryId?: number | undefined;
  settings: GenerationSettingsDto;
};

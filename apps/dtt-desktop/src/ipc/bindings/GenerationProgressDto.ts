import type { GenerationStageDto } from "./GenerationStageDto";

export type GenerationProgressDto = {
  type: "stage_changed";
  stage: GenerationStageDto;
  percent: number;
};

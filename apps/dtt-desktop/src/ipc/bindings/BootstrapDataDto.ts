import type { AppError } from "./AppError";
import type { DetectedEnvironmentDto } from "./DetectedEnvironmentDto";

export type BootstrapDataDto = {
  environment: DetectedEnvironmentDto;
  environmentError?: AppError | undefined;
  outputDirectory: string;
};

import type { ErrorCode } from "./ErrorCode";
import type { ErrorContextDto } from "./ErrorContextDto";

export type AppError = {
  code: ErrorCode;
  message: string;
  detail?: string | undefined;
  context?: ErrorContextDto | undefined;
};

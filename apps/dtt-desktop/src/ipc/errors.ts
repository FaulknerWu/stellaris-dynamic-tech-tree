import type { AppError, ErrorCode } from "./bindings";

const KNOWN_ERROR_CODES: Set<ErrorCode> = new Set([
  "INVALID_ENVIRONMENT",
  "SAVE_UNAVAILABLE",
  "SAVE_CONTAINER_CORRUPT",
  "UNSUPPORTED_BINARY_SAVE",
  "PLAYER_COUNTRY_MISSING",
  "PLAYER_COUNTRY_REQUIRED",
  "LAUNCHER_DATABASE_UNAVAILABLE",
  "GENERATION_BUSY",
  "GENERATION_CANCELLED",
  "INTERNAL",
]);

function isAppError(value: unknown): value is AppError {
  if (!value || typeof value !== "object") {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return (
    typeof candidate.code === "string" &&
    KNOWN_ERROR_CODES.has(candidate.code as ErrorCode) &&
    typeof candidate.message === "string"
  );
}

export function normalizeError(error: unknown, fallbackMessage = "发生未知错误"): AppError {
  if (isAppError(error)) {
    return error;
  }
  if (error instanceof Error) {
    return {
      code: "INTERNAL",
      message: error.message || fallbackMessage,
      detail: error.stack,
    };
  }
  if (typeof error === "string") {
    return {
      code: "INTERNAL",
      message: error,
    };
  }
  return {
    code: "INTERNAL",
    message: fallbackMessage,
    detail: JSON.stringify(error),
  };
}

import type { AppError, ErrorCode } from "./bindings";
const codes = {
 MISSING_SETTING: true,
 INVALID_PATH: true,
 NO_OUTPUT_LANGUAGE: true,
 EXECUTABLE_PATH_UNAVAILABLE: true,
 NO_TECHNOLOGY_DEFINITIONS: true,

 SAVE_UNAVAILABLE: true, SAVE_CONTAINER_CORRUPT: true,
 UNSUPPORTED_BINARY_SAVE: true, PLAYER_COUNTRY_MISSING: true, PLAYER_COUNTRY_REQUIRED: true,
 LAUNCHER_DATABASE_UNAVAILABLE: true, GENERATION_BUSY: true, GENERATION_CANCELLED: true, INTERNAL: true,
} satisfies Record<ErrorCode, boolean>;
function detail(value: unknown): string {
  try { return value instanceof Error ? value.stack ?? value.message : typeof value === "string" ? value : JSON.stringify(value) ?? String(value); }
  catch { return "Unserializable error payload"; }
}
export function normalizeError(error: unknown): AppError {
  try {
    if (error && typeof error === "object") {
      const value = error as Record<string, unknown>;
      const context = value.context;
      if (typeof value.code === "string" && Object.hasOwn(codes, value.code) &&
          (value.technicalDetail === undefined || typeof value.technicalDetail === "string") &&
          (context === undefined || context === null || typeof context === "object" &&
           (!Object.hasOwn(context, "path") || typeof (context as Record<string, unknown>).path === "string"))) {
        return { code: value.code as ErrorCode,
          ...(typeof value.technicalDetail === "string" ? { technicalDetail: value.technicalDetail } : {}),
          ...(context && typeof context === "object" && typeof (context as Record<string, unknown>).path === "string" ? { context: { path: (context as { path: string }).path } } : {}),
        };
      }
    }
  } catch { /* Hostile or corrupt payloads must not cause a second error. */ }
  return { code: "INTERNAL", technicalDetail: detail(error) };
}

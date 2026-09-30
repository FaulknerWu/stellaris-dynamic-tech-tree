import { beforeAll, expect, it, vi } from "vitest";
import { initializeI18n } from "@/i18n";
import { formatDate, formatFileSize } from "./format";

vi.mock("@/i18n/controller", () => ({ currentLocale: () => "en" }));

beforeAll(async () => {
  await initializeI18n("en");
});

it("distinguishes the Unix epoch from missing or invalid dates", () => {
  expect(formatDate(0)).not.toBe("No date");
  expect(formatDate(null)).toBe("No date");
  expect(formatDate(1e30)).toBe("No date");
});

it("uses IEC units for file sizes", () => {
  expect(formatFileSize(1024)).toBe("1 KiB");
  expect(formatFileSize(1048576)).toBe("1 MiB");
});

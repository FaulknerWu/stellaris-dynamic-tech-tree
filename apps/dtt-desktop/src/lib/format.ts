import i18n from "@/i18n";
import { currentLocale } from "@/i18n/controller";
import { numberFormatter, dateFormatter } from "@/i18n/format";
export function formatFileSize(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes < 0) return "—";
  const unit = Math.min(3, Math.max(0, Math.floor(Math.log2(bytes || 1) / 10)));
  const number = numberFormatter(currentLocale(), { maximumFractionDigits: unit === 3 ? 2 : unit ? 1 : 0 });
  return `${number.format(bytes / 1024 ** unit)} ${["B", "KiB", "MiB", "GiB"][unit]}`;
}
export function formatDate(timestampMillis: number | null): string {
  if (timestampMillis === null || !Number.isFinite(timestampMillis)) return i18n.t($ => $.noDate);
  const date = new Date(timestampMillis);
  if (!Number.isFinite(date.getTime())) return i18n.t($ => $.noDate);
  return dateFormatter(currentLocale(), { year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", second: "2-digit" }).format(date);
}

export function truncateMiddle(str: string, maxLength = 48): string {
  if (!str || str.length <= maxLength) {
    return str;
  }
  const half = Math.floor((maxLength - 3) / 2);
  return `${str.slice(0, half)}...${str.slice(-half)}`;
}

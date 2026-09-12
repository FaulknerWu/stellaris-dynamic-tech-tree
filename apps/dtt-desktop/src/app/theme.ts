import type { ThemePreference } from "./store";

let mediaQueryListener: ((e: MediaQueryListEvent) => void) | null = null;

export function applyTheme(theme: ThemePreference): void {
  const root = document.documentElement;

  if (mediaQueryListener) {
    window.matchMedia("(prefers-color-scheme: dark)").removeEventListener("change", mediaQueryListener);
    mediaQueryListener = null;
  }

  if (theme === "system") {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const applySystem = (matchesDark: boolean) => {
      if (matchesDark) {
        root.classList.add("dark");
      } else {
        root.classList.remove("dark");
      }
    };
    applySystem(mediaQuery.matches);
    mediaQueryListener = (e) => applySystem(e.matches);
    mediaQuery.addEventListener("change", mediaQueryListener);
  } else if (theme === "dark") {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }
}

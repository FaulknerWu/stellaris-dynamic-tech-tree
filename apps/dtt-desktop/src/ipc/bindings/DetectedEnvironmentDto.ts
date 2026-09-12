export type DetectedEnvironmentDto = {
  gameRoot?: string | undefined;
  documentsDir?: string | undefined;
  launcherDb?: string | undefined;
  steamLibraries: Array<string>;
};
